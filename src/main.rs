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

//use serde_json::Value;
use anyhow::Result;
use structopt::StructOpt;
#[allow(unused_imports)]
use tokio::prelude::*;

mod security;
mod quic;

#[path = "./options/mod.rs"]
mod options;

#[macro_use]
extern crate lazy_static;

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

use broker_proto::bl;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Human {
    age: u32,
    name: String,
    gender: Option<String>,
}

lazy_static! {
    static ref OPT: options::Opt = options::Opt::from_args();
}

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish(),
    ).unwrap();

    let lco =  None::<bl::ListContainersOptions<String>>;

    let list_load = serde_json::to_string_pretty(&lco).unwrap();
    println!("{}", list_load);
   
    let code = {
        if let Err(e) = run() {
            eprintln!("ERROR: {}", e);
            1
        } else {
            0
        }
    };

    std::process::exit(code);
}

use warp::Filter;
use warp::path::path;
use warp::path::param;
use warp::body::json;

#[tokio::main]
async fn run() -> Result<()> {
    let list = path("list")
        .and(json())
        .and_then(list);

    let inspect = param::<String>()
        .and(path("inspect"))    
        .and(json())
        .and_then(inspect);

    let prune = path("prune")
        .and(json())
        .and_then(prune);

    let create = path("create")
        .and(json())
        .and(json())
        .and_then(create);

    let change = param::<String>()
        .and(path("change"))
        .and_then(change);

    let logs = param::<String>()
        .and(path("logs"))
        .and(json())
        .and_then(logs);

    let stats = param::<String>()
        .and(path("stats"))
        .and(json())
        .and_then(stats);
    
    let stop = param::<String>()
        .and(path("stop"))
        .and(json())
        .and_then(stop);

    let start = param::<String>()
        .and(path("start"))
        .and(json())
        .and_then(start);

    let kill = param::<String>()
        .and(path("kill"))
        .and(json())
        .and_then(kill);

    let restart = param::<String>()
        .and(path("restart"))
        .and(json())
        .and_then(restart);

    let top = param::<String>()
        .and(path("top"))
        .and(json())
        .and_then(top);

    let remove = param::<String>()
        .and(path("remove"))
        .and(json())
        .and_then(remove);

    let update = param::<String>()
        .and(path("update"))
        .and(json())
        .and_then(update);

    let promote = warp::post().and(path("container").and(inspect.or(create)
        .or(change).or(logs).or(stats).or(stop).or(start).or(kill).or(restart).or(top).or(remove).or(update).or(list).or(prune)));

    warp::serve(promote).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

use broker_proto::Protocol;
use std::convert::Infallible;
use warp::reply::Json;

async fn list(opt: Option<bl::ListContainersOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::list_opt(opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn inspect(name: String, opt: Option<bl::InspectContainerOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::inspect_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn prune(opt: Option<bl::PruneContainersOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::prune_opt(opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn create(config: bl::Config<String>, opt: Option<bl::CreateContainerOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::create_opt(config, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn change(name: String) -> Result<Json, Infallible> {
    let proto = Protocol::change_opt(&name).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn logs(name: String, opt: Option<bl::LogsOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::logs_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn stats(name: String, opt: Option<bl::StatsOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::stats_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn stop(name: String, opt: Option<bl::StopContainerOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::stop_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn start(name: String, opt: Option<bl::StartContainerOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::start_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn kill(name: String, opt: Option<bl::KillContainerOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::kill_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn restart(name: String, opt: Option<bl::RestartContainerOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::restart_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn top(name: String, opt: Option<bl::TopOptions<String>>) -> Result<Json, Infallible> {
    let proto = Protocol::top_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn remove(name: String, opt: Option<bl::RemoveContainerOptions>) -> Result<Json, Infallible> {
    let proto = Protocol::remove_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}

async fn update(name: String, opt: bl::UpdateContainerOptions) -> Result<Json, Infallible> {
    let proto = Protocol::update_opt(&name, opt).await;
    if let Err(_) = proto {
        return Ok(warp::reply::json(&Protocol::error_none("error")));
    }

    let resp = quic::handle_request(&OPT, proto.unwrap()).await;
    if let Err(_) = resp {
        return Ok(warp::reply::json(&Protocol::error_none("error2")));
    }
    Ok(warp::reply::json(&resp.unwrap()))
}