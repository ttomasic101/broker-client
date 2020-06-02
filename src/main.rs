#![type_length_limit = "2097152"]
#[allow(unused_imports)]
use std::{
    fs,
    io::{self, Write, Cursor},
    net::{ToSocketAddrs, SocketAddr},
    path::PathBuf,
    time::{Duration, Instant},
};

extern crate common;
use broker_proto::Protocol;

//use serde_json::Value;
use anyhow::{anyhow, Result};
use structopt::StructOpt;
use tracing::{error, info};
#[allow(unused_imports)]
use tokio::prelude::*;

mod security;
mod quic;
mod options;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use rmps::{Deserializer, Serializer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Human {
    age: u32,
    name: String,
    gender: Option<String>,
}

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish(),
    ).unwrap();

    let opt = options::Opt::from_args();
    let code = {
        if let Err(e) = run(&opt) {
            eprintln!("ERROR: {}", e);
            1
        } else {
            0
        }
    };

    std::process::exit(code);
}

#[tokio::main]
async fn run(options: &options::Opt) -> Result<()> {
    let mut endpoint = quinn::Endpoint::builder();
    let mut client_config = quinn::ClientConfigBuilder::default();
    client_config.protocols(common::ALPN_QUIC_HTTP);

    if options.keylog {
        client_config.enable_keylog();
    }

    security::setup_security(&mut client_config, &options.ca)?;

    /*
    if let Some(ca_path) = options.ca {
        client_config
            .add_certificate_authority(quinn::Certificate::from_der(&fs::read(&ca_path)?)?)?;
    } else {
        let dirs = directories::ProjectDirs::from("arg", "quinn", "quinn-examples").unwrap();
        match fs::read(dirs.data_local_dir().join("cert.der")) {
            Ok(cert) => {
                client_config.add_certificate_authority(quinn::Certificate::from_der(&cert)?)?;
            },
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("Local server certificate not found");
            },
            Err(e) => {
                error!("Failed to open local server certificate: {}", e);
            }
        }
    }
    */

    endpoint.default_client_config(client_config.build());

    let (endpoint, _) = endpoint.bind(&"0.0.0.0:0".parse()?)?;

    let test = broker_proto::Protocol::list().await?;
    let mut buf = Vec::new();

    test.serialize(&mut Serializer::new(&mut buf))?;

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

    send.write_all(&test.bytes().unwrap())
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

    use std::convert::TryFrom;
    let test: Protocol = broker_proto::Protocol::try_from(&resp[..])?;
    let tmp = format!("{:#?}", test);
    
    
    io::stdout().write_all(tmp.as_bytes()).unwrap();
    io::stdout().flush().unwrap();
    conn.close(0u32.into(), b"done");

    Ok(())
}

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}