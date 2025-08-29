extern crate alloc;

mod error;
mod utils;

use std::net::{
    TcpListener,
    TcpStream,
};
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use codec::dec::Decode as _;
use data::model::{
    handshake,
    login,
};
use data::packet::{
    ReadPacket as _,
    WritePacket as _,
};
use log::{
    debug,
    error,
    info,
    trace,
};

use crate::error::Error;

#[derive(Parser, Debug)]
#[command(about, version, author)]
struct Cli {
    #[arg(long, env, default_value = "35565")]
    proxy_port: u16,
    #[arg(long, env)]
    server_host: String,
    #[arg(long, env, default_value = "25565")]
    server_port: u16,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    let proxy_addr = format!(
        "{proxy_host}:{proxy_port}",
        proxy_host = "0.0.0.0",
        proxy_port = args.proxy_port
    );
    let server_addr = format!(
        "{server_host}:{server_port}",
        server_host = args.server_host,
        server_port = args.server_port
    );

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

#[derive(Debug)]
struct ConnectionState {
    pub stage: ConnectionStage,
    pub packet_min_compression: Option<usize>,
}

#[derive(Debug)]
enum ConnectionStage {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
    End,
}

fn handle_connection(
    mut client: TcpStream,
    mut server: TcpStream,
) -> Result<(), Error> {
    _ = client.set_nodelay(true);
    _ = server.set_nodelay(true);

    let mut connection_state = ConnectionState {
        stage: ConnectionStage::Handshake,
        packet_min_compression: None,
    };

    loop {
        match connection_state.stage {
            ConnectionStage::Handshake => {
                handle_handshake(&mut client, &mut server, &mut connection_state)?;
            }
            ConnectionStage::Status => {
                handle_status(&mut client, &mut server, &mut connection_state)?;
            }
            ConnectionStage::Login => {
                handle_login(&mut client, &mut server, &mut connection_state)?;
            }
            ConnectionStage::Configuration => {
                // TODO: parse configuration packets
                break;
            }
            ConnectionStage::Play => {
                // TODO: parse play packets
                break;
            }
            ConnectionStage::End => return Ok(()),
        }
    }

    // Pump remaining data between client and server
    let client_read = client.try_clone().map_err(Error::TcpStreamClone)?;
    let server_read = server.try_clone().map_err(Error::TcpStreamClone)?;

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || {
        tx1.send(relay(
            Relay::ClientToServer,
            client_read,
            server,
            connection_state.packet_min_compression,
        ))
    });
    thread::spawn(move || {
        tx2.send(relay(
            Relay::ServerToClient,
            client,
            server_read,
            connection_state.packet_min_compression,
        ))
    });

    rx1.recv()??;
    rx2.recv()??;

    Ok(())
}

fn handle_handshake(
    client: &mut TcpStream,
    server: &mut TcpStream,
    state: &mut ConnectionState,
) -> Result<(), Error> {
    // 0x00 intention
    let packet = client.read_packet(state.packet_min_compression.is_some())?;
    server.write_packet(&packet, state.packet_min_compression)?;

    let handshake = handshake::Handshake::decode(&mut packet.data.as_ref())?;
    debug!("{handshake:?}");

    match handshake.intent {
        handshake::Intent::Status => state.stage = ConnectionStage::Status,
        handshake::Intent::Login => state.stage = ConnectionStage::Login,
        handshake::Intent::Transfer => {
            // after a transfer command from the Configuration or Play state, we should
            // recieve a handshake with the Transfer intent, and then a login packet
            state.stage = ConnectionStage::Login;
        }
    }

    Ok(())
}

fn handle_status(
    client: &mut TcpStream,
    server: &mut TcpStream,
    state: &mut ConnectionState,
) -> Result<(), Error> {
    // 0x00 status_request
    let packet = client.read_packet(state.packet_min_compression.is_some())?;
    server.write_packet(&packet, state.packet_min_compression)?;

    // 0x00 status_response
    let packet = server.read_packet(state.packet_min_compression.is_some())?;
    let packet = crate::utils::inject_status_description_message(&packet)?;
    client.write_packet(&packet, state.packet_min_compression)?;

    // 0x01 ping_request
    let packet = client.read_packet(state.packet_min_compression.is_some())?;
    server.write_packet(&packet, state.packet_min_compression)?;

    // 0x01 pong_response
    let packet = server.read_packet(state.packet_min_compression.is_some())?;
    client.write_packet(&packet, state.packet_min_compression)?;

    state.stage = ConnectionStage::End;

    Ok(())
}

fn handle_login(
    client: &mut TcpStream,
    server: &mut TcpStream,
    state: &mut ConnectionState,
) -> Result<(), Error> {
    // 0x00 hello
    let packet = client.read_packet(state.packet_min_compression.is_some())?;
    server.write_packet(&packet, state.packet_min_compression)?;

    let hello = login::Hello::decode(&mut packet.data.as_ref())?;
    debug!("{hello:?}");

    loop {
        let packet = server.read_packet(state.packet_min_compression.is_some())?;
        client.write_packet(&packet, state.packet_min_compression)?;

        let mut data = packet.data.as_ref();

        match packet.id {
            // 0x00 login_disconnect
            0x00 => {
                trace!("{state:?}: Received from server: 0x00 login_disconnect");
                state.stage = ConnectionStage::End;
                break;
            }
            // 0x01 hello (set encryption)
            0x01 => {
                trace!("{state:?}: Received from server: 0x01 hello");
                // 0x01 key
                let packet = client.read_packet(state.packet_min_compression.is_some())?;
                server.write_packet(&packet, state.packet_min_compression)?;
                trace!("{state:?}: Sent to server: 0x01 key");
            }
            // 0x02 login_finished
            0x02 => {
                trace!("{state:?}: Received from server: 0x02 login_finished");
                // 0x03 login_acknowledged
                let packet = client.read_packet(state.packet_min_compression.is_some())?;
                server.write_packet(&packet, state.packet_min_compression)?;
                trace!("{state:?}: Sent to server: 0x03 login_acknowledged");

                state.stage = ConnectionStage::Configuration;
                break;
            }
            // 0x03 login_compression (set compression)
            0x03 => {
                trace!("{state:?}: Received from server: 0x03 login_compression");
                let login_compression = login::LoginCompression::decode(&mut data)?;

                if login_compression.size >= 0 {
                    let min_packet_compression = login_compression.size.cast_unsigned() as usize;
                    state.packet_min_compression = Some(min_packet_compression);
                } else {
                    state.packet_min_compression = None;
                }
            }
            // 0x04 custom_query
            0x04 => {
                trace!("{state:?}: Received from server: 0x04 custom_query");
                // 0x02 custom_query_answer
                let packet = client.read_packet(state.packet_min_compression.is_some())?;
                server.write_packet(&packet, state.packet_min_compression)?;
                trace!("{state:?}: Sent to server: 0x02 custom_query_answer");
            }
            // 0x05 cookie_request
            0x05 => {
                trace!("{state:?}: Received from server: 0x05 cookie_request");
                // 0x04 cookie_response
                let packet = client.read_packet(state.packet_min_compression.is_some())?;
                server.write_packet(&packet, state.packet_min_compression)?;
                trace!("{state:?}: Sent to server: 0x04 cookie_response");
            }
            _ => return Err(Error::UnknownPacketId(packet.id)),
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Relay {
    ClientToServer,
    ServerToClient,
}

fn relay(
    relay: Relay,
    mut client: TcpStream,
    mut server: TcpStream,
    min_compression: Option<usize>,
) -> Result<(), Error> {
    let (from, to) = match relay {
        Relay::ClientToServer => (&mut client, &mut server),
        Relay::ServerToClient => (&mut server, &mut client),
    };

    loop {
        // Check if EOF has been reached
        if let Ok(n) = from.peek(&mut [0_u8])
            && n == 0
        {
            // Connection closed
            return Ok(());
        }

        let packet = from.read_packet(min_compression.is_some())?;
        to.write_packet(&packet, min_compression)?;

        debug!("{relay:?} {packet:?}");
    }
}
