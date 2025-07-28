extern crate alloc;

mod error;

mod handshake;
mod login;
mod status;

use std::env;
use std::net::TcpListener;

use data::model::Intent;

fn main() {
    let server_host = "0.0.0.0";
    let server_port = env::var("SERVER_PORT").unwrap_or("25565".to_owned());
    let server_addr = format!("{server_host}:{server_port}");

    let listener = TcpListener::bind(server_addr).expect("Failed to bind to address");

    for mut stream in listener.incoming().flatten() {
        let handshake = match handshake::process(&mut stream) {
            Ok(handshake) => handshake,
            Err(err) => {
                eprintln!("Error handling handshake: {err}");
                continue;
            }
        };

        match handshake.intent {
            Intent::Status => {
                if let Err(err) = status::process(&mut stream) {
                    eprintln!("Error handling status: {err}");
                }
            }
            Intent::Login => {
                if let Err(err) = login::process(&mut stream) {
                    eprintln!("Error handling login: {err}");
                }
            }
            Intent::Transfer => todo!("handle transfer intent"),
        }
    }
}
