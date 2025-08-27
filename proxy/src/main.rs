extern crate alloc;

use std::net::{
    Shutdown,
    TcpListener,
    TcpStream,
};
use std::{
    env,
    io,
    thread,
};

fn main() {
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
                eprintln!("Failed to accept client connection: {err}");
                continue;
            }
        };

        let server = match TcpStream::connect(&server_addr) {
            Ok(server) => server,
            Err(err) => {
                eprintln!("Failed to connect to server: {err}");
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
            relay(client_read, server);
        });

        thread::spawn(move || {
            relay(server_read, client);
        });
    }
}

fn relay(
    mut from: TcpStream,
    mut to: TcpStream,
) {
    _ = io::copy(&mut from, &mut to);
    _ = to.shutdown(Shutdown::Write);
}
