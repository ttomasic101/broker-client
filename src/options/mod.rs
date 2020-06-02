use structopt::StructOpt;
use std::{
    net::SocketAddr,
    path::PathBuf
};

#[derive(StructOpt, Debug)]
#[structopt(name = "client")]
pub struct Opt {

    #[structopt(long = "keylog")]
    pub keylog: bool,

    #[structopt(long = "server", default_value = "127.0.0.1:8000")]
    pub server: SocketAddr,

    #[structopt(long = "host", default_value = "localhost")]
    pub host: String,

    #[structopt(parse(from_os_str), long = "ca")]
    pub ca: Option<PathBuf>,

    #[structopt(long = "rebind")]
    pub rebind: bool,
}