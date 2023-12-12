// main.rs

mod server;
mod cli;

use server::run_server;

#[tokio::main]
async fn main() {
    run_server().await.unwrap();
}
