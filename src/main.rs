mod client;
mod movement;
mod protocol;
mod server;
mod shared;
mod world;

use crate::client::{build_client_net_config, client_app};
use crate::server::{build_server_net_config, server_app};
use clap::Parser;
use std::env;

#[derive(Parser, Clone, Copy)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    listen_server: bool,

    #[arg(short, long, default_value_t = 1234)]
    client_id: u64,
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    let cli = Cli::parse();

    println!("listen_server: {:?}", cli.listen_server);
    println!("client_id: {:?}", cli.client_id);

    let mut client_app = client_app(build_client_net_config(cli.client_id, "127.0.0.1:5000"));

    if cli.listen_server {
        let mut server_app = server_app(build_server_net_config());

        std::thread::spawn(move || server_app.run());
    }

    client_app.run();
}
