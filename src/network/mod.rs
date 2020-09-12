mod ack_bitfield;
mod client;
mod connection;
mod server;

pub use client::Client;
use std::convert::TryInto;
use thiserror::Error;

// TODO: ideally this should be versioned
// one possibility would to use some kind of hash of the version/commit
type ProtocolId = [u8; 4];
const PROTOCOL_ID: ProtocolId = [0u8, 1, 2, 3];

// Ethernet MTU - IP header - UDP header
const MAX_PACKET_SIZE: usize = 1500 - 20 - 8;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Invalid protocol id")]
    InvalidProtocolId,
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("can't send message")]
    ChannelSendError(#[from] futures::channel::mpsc::SendError),
}

pub type Result<T> = std::result::Result<T, NetworkError>;

pub struct Packet<B>(pub B);
pub struct PacketHeader<B>(pub B);

impl Packet<Box<[u8]>> {
    pub fn new_boxed(payload_len: usize) -> Self {
        let buf = vec![0u8; payload_len + connection::header_size()];

        Self(buf.into_boxed_slice())
    }
}

impl<B: AsRef<[u8]>> Packet<B> {
    pub fn header(&self) -> PacketHeader<&[u8]> {
        use std::mem::{size_of, size_of_val};
        PacketHeader(
            &self.0.as_ref()[0..size_of_val(&PROTOCOL_ID)
                + 2usize * size_of::<connection::SeqNumber>()
                + size_of::<ack_bitfield::AckBitfield>()],
        )
    }

    pub fn payload(&self) -> &[u8] {
        use std::mem::{size_of, size_of_val};
        &self.0.as_ref()[size_of_val(&PROTOCOL_ID)
            + 2usize * size_of::<connection::SeqNumber>()
            + size_of::<ack_bitfield::AckBitfield>()..]
    }

    pub fn into_inner(self) -> B {
        self.0
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> Packet<B> {
    fn header_mut(&mut self) -> PacketHeader<&mut [u8]> {
        use std::mem::{size_of, size_of_val};
        PacketHeader(
            &mut self.0.as_mut()[0..size_of_val(&PROTOCOL_ID)
                + 2usize * size_of::<connection::SeqNumber>()
                + size_of::<ack_bitfield::AckBitfield>()],
        )
    }

    pub fn payload_mut(&mut self) -> &mut [u8] {
        use std::mem::{size_of, size_of_val};
        &mut self.0.as_mut()[size_of_val(&PROTOCOL_ID)
            + 2usize * size_of::<connection::SeqNumber>()
            + size_of::<ack_bitfield::AckBitfield>()..]
    }
}

impl<T> PacketHeader<T> {
    pub const fn len() -> usize {
        use std::mem::*;
        size_of::<ProtocolId>()
            + size_of::<connection::SeqNumber>() * 2
            + size_of::<ack_bitfield::AckBitfield>()
    }
}

impl<B: AsRef<[u8]>> PacketHeader<B> {
    fn protocol_id(&self) -> [u8; 4] {
        self.0.as_ref()[0..std::mem::size_of_val(&PROTOCOL_ID)]
            .try_into()
            .unwrap()
    }

    fn seq(&self) -> connection::SeqNumber {
        let low: usize = std::mem::size_of_val(&PROTOCOL_ID);

        connection::SeqNumber::from_bytes(
            &self.0.as_ref()[low..low + std::mem::size_of::<connection::SeqNumber>()],
        )
    }

    fn ack(&self) -> connection::SeqNumber {
        let low: usize =
            std::mem::size_of_val(&PROTOCOL_ID) + std::mem::size_of::<connection::SeqNumber>();

        connection::SeqNumber::from_bytes(
            &self.0.as_ref()[low..low + std::mem::size_of::<connection::SeqNumber>()],
        )
    }

    fn ack_bitfield(&self) -> ack_bitfield::AckBitfield {
        let low: usize =
            std::mem::size_of_val(&PROTOCOL_ID) + 2 * std::mem::size_of::<connection::SeqNumber>();

        ack_bitfield::AckBitfield::from_bytes(
            &self.0.as_ref()[low..low + std::mem::size_of::<ack_bitfield::AckBitfield>()],
        )
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> PacketHeader<B> {
    fn set(
        &mut self,
        seq: connection::SeqNumber,
        ack: connection::SeqNumber,
        ack_bitfield: ack_bitfield::AckBitfield,
    ) {
        let buf = self.0.as_mut();

        let mut low = 0;

        let parts = [
            PROTOCOL_ID,
            seq.to_bytes(),
            ack.to_bytes(),
            ack_bitfield.to_bytes(),
        ];

        for part in parts.iter() {
            buf[low..low + part.len()].copy_from_slice(part);
            low += part.len();
        }
    }
}
