extern crate simple_platformer;
#[macro_use]
extern crate log;
use clap::{App, Arg};
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};
use log::{error, info};
use simple_platformer::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const DEFAULT_PORT: u32 = 6666;

fn main() {
    env_logger::init();

    let matches = App::new("battle arena 2000 - server")
        .version("0.0.1-alpha")
        .author("Enzo Cioppettini <hi@enzocioppettini.com>")
        .about("Push people out")
        .arg(Arg::new("PORT").short('p').long("port").takes_value(true))
        .get_matches();

    let (tx, rx) = channel();

    let port = matches
        .value_of("PORT")
        .map(|port| port.parse::<_>().expect("port is not a number"))
        .unwrap_or(DEFAULT_PORT);

    thread::spawn(move || {
        info!("connection handler listening on port {}", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                info!("accepted stream");
                tx.send(stream).unwrap();
            } else {
                error!("failed to accept conection");
            }
        }
    });

    let mut server = server::Server::new();
    let mut last_instant = std::time::Instant::now();

    loop {
        let new_instant = std::time::Instant::now();
        let dt = new_instant - last_instant;
        last_instant = new_instant;
        server.process_client_messages();
        server.update_state(dt);

        let client = rx.recv_timeout(std::time::Duration::from_millis(10));

        match client {
            Ok(client) => {
                server.add_client(client);
            }
            Err(err) => {}
        }
    }
}
