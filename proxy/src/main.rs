extern crate alloc;

mod error;

use std::net::{
    TcpListener,
    TcpStream,
};
use std::{
    env,
    io,
    thread,
};

use data::packet::{
    ReadPacket as _,
    WritePacket as _,
};
use log::{
    debug,
    error,
    info,
};

use crate::error::Error;

fn main() {
    env_logger::init();

    let proxy_host = "0.0.0.0";
    let proxy_port = env::var("PROXY_PORT").unwrap_or("35565".to_owned());
    let proxy_addr = format!("{proxy_host}:{proxy_port}");

    let server_host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_owned());
    let server_port = env::var("SERVER_PORT").unwrap_or("25565".to_owned());
    let server_addr = format!("{server_host}:{server_port}");

    let listener = TcpListener::bind(proxy_addr).expect("Failed to bind to proxy address");

    for client in listener.incoming() {
        let client = match client {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to accept client connection: {err}");
                continue;
            }
        };

        if let Ok(client_addr) = client.peer_addr() {
            info!("Accepted client connection from {client_addr}",);
        }

        let server = match TcpStream::connect(&server_addr) {
            Ok(server) => server,
            Err(err) => {
                error!("Failed to connect to server: {err}");
                continue;
            }
        };

        let client_read = client
            .try_clone()
            .expect("Failed to clone client tcp stream");
        let server_read = server
            .try_clone()
            .expect("Failed to clone server tcp stream");

        thread::spawn(move || {
            match relay(client_read, server) {
                Err(Error::Encode(err))
                    if err
                        .get_io_error()
                        .filter(|err| err.kind() == io::ErrorKind::BrokenPipe)
                        .is_some() =>
                {
                    // Ignore broken pipe errors
                }
                Err(err) => error!("Failed to relay client to server: {err}"),
                _ => {}
            }
        });

        thread::spawn(move || {
            match relay(server_read, client) {
                Err(Error::Encode(err))
                    if err
                        .get_io_error()
                        .filter(|err| err.kind() == io::ErrorKind::BrokenPipe)
                        .is_some() =>
                {
                    // Ignore broken pipe errors
                }
                Err(err) => error!("Failed to relay server to client: {err}"),
                _ => {}
            }
        });
    }
}

fn relay(
    mut from: TcpStream,
    mut to: TcpStream,
) -> Result<(), Error> {
    loop {
        // Check if EOF has been reached
        if let Ok(n) = from.peek(&mut [0_u8])
            && n == 0
        {
            // Connection closed
            return Ok(());
        }

        let packet = from.read_packet()?;

        debug!("{packet:?}");

        to.write_packet(&packet)?;
    }
}
