use super::connection::Connection;
use super::{NetworkError, Packet, MAX_PACKET_SIZE};
use async_std::net::{ToSocketAddrs, UdpSocket};
use async_std::sync::{Arc, Mutex};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::task::Poll;
use futures::{future::FutureExt, pin_mut, task::Context, Sink, Stream, StreamExt};
use log::{debug, error};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;

#[pin_project]
pub struct Client<B> {
    connection: Arc<Mutex<Connection>>,
    socket: UdpSocket,
    #[pin]
    sender: UnboundedSender<Packet<B>>,
    #[pin]
    receiver: UnboundedReceiver<Packet<B>>,
    flush_counter: std::sync::atomic::AtomicUsize,
}

async fn send_packet<B: AsRef<[u8]>>(socket: &UdpSocket, bytes: B) -> Result<(), NetworkError> {
    debug!("sending packet");

    // FIXME: Remove this assert_eq
    socket
        .send(bytes.as_ref())
        .await
        .map(|written| assert_eq!(bytes.as_ref().len(), written))
        .map_err(NetworkError::from)
}

impl<B> Client<B> {
    pub async fn new(
        bind_address: impl ToSocketAddrs,
        server_address: impl ToSocketAddrs,
    ) -> Result<Self, NetworkError> {
        let connection = Arc::new(Mutex::new(Connection::new()));

        let socket = UdpSocket::bind(bind_address).await?;
        socket.connect(server_address).await?;

        let (sender, receiver) = unbounded();

        let flush_counter = Default::default();
        Ok(Client {
            connection,
            socket,
            sender,
            receiver,
            flush_counter,
        })
    }

    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.connection)
    }

    pub async fn update(&self, dt: std::time::Duration) {
        self.connection.lock().await.update(dt);
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> Sink<Packet<B>> for Client<B> {
    type Error = NetworkError;
    fn start_send(mut self: Pin<&mut Self>, packet: Packet<B>) -> Result<(), Self::Error> {
        debug!("start send");
        // TODO: may be relaxed?
        self.flush_counter
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        self.sender.start_send(packet).map_err(Into::into)
    }

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        debug!("polling inner sender");
        match self.sender.poll_ready(cx) {
            Poll::Ready(Ok(())) => (),
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
        };

        debug!("polling inner receiver");

        let buffered_messages = self.flush_counter.load(Ordering::Acquire);

        if buffered_messages > 0 {
            match self.receiver.poll_next_unpin(cx) {
                Poll::Ready(Some(mut packet)) => {
                    debug!("increasing counter");
                    self.flush_counter
                        .fetch_sub(1, std::sync::atomic::Ordering::Release);
                    Future::poll(
                        Box::pin(
                            async {
                                debug!("locking connection");
                                self.connection
                                    .lock()
                                    .await
                                    .fill_header(packet.header_mut());

                                debug!("header filled");
                                packet
                            }
                            .then(|packet| {
                                let packet = packet.into_inner();
                                send_packet(&self.socket, packet)
                            }),
                        )
                        .as_mut(),
                        cx,
                    )
                }
                Poll::Pending => {
                    debug!("receiver is pending");
                    Poll::Pending
                }
                Poll::Ready(None) => unreachable!("the stream shouldn't really end"),
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let sender = this.sender;
        let connection = this.connection;
        let mut receiver = this.receiver;
        let socket = this.socket;
        let flush_counter = this.flush_counter;

        match sender.poll_flush(cx) {
            Poll::Ready(Ok(())) => (),
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
        };

        let buffered_messages = flush_counter.load(Ordering::Acquire);

        for _ in 0..buffered_messages {
            match receiver.poll_next_unpin(cx) {
                Poll::Ready(Some(mut packet)) => {
                    match Future::poll(
                        Box::pin(
                            async {
                                connection.lock().await.fill_header(packet.header_mut());
                                packet
                            }
                            .then(|packet| {
                                let packet = packet.into_inner();
                                send_packet(&socket, packet)
                            }),
                        )
                        .as_mut(),
                        cx,
                    ) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(_)) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                    }
                }
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => unreachable!("the stream shouldn't really end"),
            }
        }

        flush_counter.store(0, Ordering::Release);

        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

impl<B> Stream for Client<B> {
    type Item = Packet<Box<[u8]>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let fut = async {
            let mut buffer = vec![0u8; MAX_PACKET_SIZE];
            let read = self.socket.recv(&mut buffer).await.unwrap();
            unsafe { buffer.set_len(read) };

            let packet = Packet(buffer.into_boxed_slice());

            if let Err(e) = self.connection.lock().await.check(&packet) {
                error!("connection receive error: {}", e);
                panic!();
            }

            Some(packet)
        };

        pin_mut!(fut);

        fut.poll(cx)
    }
}
