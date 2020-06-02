use anyhow::{Result, anyhow};
use broker_proto::Protocol;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use rmps::{Deserializer, Serializer};

use std::{
    time::{Instant, Duration},
};

#[path = "../options/mod.rs"]
mod options;

#[path = "../security/mod.rs"]
mod security;

pub async fn handle_request(options: &options::Opt, data: Vec<u8>) -> Result<Vec<u8>> {

    let mut endpoint = quinn::Endpoint::builder();
    let mut client_config = quinn::ClientConfigBuilder::default();
    client_config.protocols(common::ALPN_QUIC_HTTP);

    if options.keylog {
        client_config.enable_keylog();
    }

    security::setup_security(&mut client_config, &options.ca)?;

    endpoint.default_client_config(client_config.build());

    let (endpoint, _) = endpoint.bind(&"0.0.0.0:0".parse()?)?;

    use std::convert::TryFrom;
    let request: Protocol = broker_proto::Protocol::try_from(&data[..])?;
    //let tmp = format!("{:#?}", test);

    let start = Instant::now();
    //let rebind = options.rebind;
    let host = options
        .host
        .as_str();
    
    let new_conn = endpoint
        .connect(&options.server, &host)?
        .await
        .map_err(|e| anyhow!("Failed to connect: {}", e))?;
    
    eprintln!("Connected as {:?}", start.elapsed());

    let quinn::NewConnection {
        connection: conn, ..
    } = {new_conn};

    let (mut send, recv) = conn
        .open_bi()
        .await
        .map_err(|e| anyhow!("Failed to open stream: {}", e))?;

    if options.rebind {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
        let addr = socket.local_addr().unwrap();
        eprintln!("Rebinding to {}", addr);
        endpoint.rebind(socket).expect("Rebind failed");
    }

    let mut buf = Vec::new();

    request.serialize(&mut Serializer::new(&mut buf))?;

    send.write_all(&buf)
        .await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;

    send.finish()
        .await
        .map_err(|e| anyhow!("Failed to shutdown stream: {}", e))?;

    let response_start = Instant::now();
    eprintln!("Request sent at {:?}", response_start - start);

    let resp = recv
        .read_to_end(usize::max_value())
        .await
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;

    
    let duration = response_start.elapsed();
    eprintln!(
        "Response received in {:?} - {} KiB/s",
        duration,
        resp.len() as f32 / (duration_secs(&duration) * 1024.0)
    );

    let test: Protocol = broker_proto::Protocol::try_from(&resp[..])?;

    let mut buf = Vec::new();
    test.serialize(&mut Serializer::new(&mut buf))?;

    Ok(buf)
}

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}