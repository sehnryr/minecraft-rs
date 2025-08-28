extern crate alloc;

mod error;

use std::net::{
    TcpListener,
    TcpStream,
};
use std::sync::mpsc;
use std::{
    env,
    thread,
};

use codec::dec::Decode as _;
use data::model::{
    Handshake,
    Intent,
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

        thread::spawn(move || {
            if let Err(err) = handle_connection(client, server) {
                error!("Failed to handle connection: {err}");
            }
        });
    }
}

enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

fn handle_connection(
    mut client: TcpStream,
    mut server: TcpStream,
) -> Result<(), Error> {
    let mut connection_state = ConnectionState::Handshake;

    loop {
        match connection_state {
            ConnectionState::Handshake => {
                // 0x00 intention
                let packet = client.read_packet()?;
                server.write_packet(&packet)?;

                let handshake = Handshake::decode(&mut packet.data.as_slice())?;
                debug!("Recieved Handshake: {handshake:?}");

                match handshake.intent {
                    Intent::Status => connection_state = ConnectionState::Status,
                    Intent::Login => connection_state = ConnectionState::Login,
                    Intent::Transfer => {
                        // after a transfer command from the Configuration or Play state, we should
                        // recieve a handshake with the Transfer intent, and then a login packet
                        connection_state = ConnectionState::Login;
                    }
                }
            }
            ConnectionState::Status => {
                // TODO: parse status packets
                break;
            }
            ConnectionState::Login => {
                // TODO: parse login packets
                break;
            }
            ConnectionState::Configuration => {
                // TODO: parse configuration packets
                break;
            }
            ConnectionState::Play => {
                // TODO: parse play packets
                break;
            }
        }
    }

    // Pump remaining data between client and server
    let client_read = client.try_clone().map_err(Error::TcpStreamClone)?;
    let server_read = server.try_clone().map_err(Error::TcpStreamClone)?;

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || tx1.send(relay(client_read, server)));
    thread::spawn(move || tx2.send(relay(server_read, client)));

    rx1.recv()??;
    rx2.recv()??;

    Ok(())
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
