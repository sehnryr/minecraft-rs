use codec::dec::Decode as _;
use codec::enc::{
    Encode as _,
    EncodeErrorContext as _,
};
use data::packet::Packet;
use json::JsonValue;
use log::debug;

use crate::error::Error;

pub fn inject_status_description_message(packet: &Packet) -> Result<Packet, Error> {
    let mut json_response = JsonValue::decode(&mut packet.data.as_ref())?;
    debug!("Recieved Status response: {json_response}");

    if json_response["description"].is_null() {
        json_response["description"] = json::object! {
            text: "proxied by minecraft-rs 🦀",
            color: "#d34516",
        };
    } else if let Some(description) = json_response["description"].as_str() {
        json_response["description"] = json::object! {
            text: description,
            extra: [
                {
                    text: "\nproxied by minecraft-rs 🦀",
                    color: "#d34516",
                }
            ]
        };
    } else if json_response["description"].is_object() {
        if json_response["description"]["extra"].is_null() {
            json_response["description"]["extra"] = json::array![
                {
                    text: "\nproxied by minecraft-rs 🦀",
                    color: "#d34516",
                }
            ];
        } else if json_response["description"]["extra"].is_array() {
            _ = json_response["description"]["extra"].push(json::object! {
                text: "\nproxied by minecraft-rs 🦀",
                color: "#d34516",
            });
        }
    }

    let mut data = Vec::new();
    json_response
        .encode(&mut data)
        .err_context("Failed to encode status response")?;

    let packet = Packet::new(packet.id, &data);

    Ok(packet)
}
