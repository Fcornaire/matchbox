use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[clap(
    name = "made_in_heaven",
    rename_all = "kebab-case",
    rename_all_env = "screaming-snake"
)]
pub struct Args {
    #[clap(default_value = "0.0.0.0:3536", env)]
    pub host: SocketAddr,

    #[clap(default_value = "0.0.0.0:3738", env)]
    pub metrics: SocketAddr,

    #[clap(long, env)]
    pub integrity_hash: Option<String>,
}
