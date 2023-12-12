// server.rs

use super::cli;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting the server...");

    let map: Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    let counter = Arc::new(Mutex::new(1));

    let listen = TcpListener::bind("0.0.0.0:9999").await?;

    tokio::spawn(cli::handle_cli_commands(Arc::clone(&map)));

    loop {
        let (mut socket, _) = listen.accept().await?;
        let (tx, mut rx) = mpsc::channel(100);
        let id = {
            let mut counter = counter.lock().unwrap();
            let id = *counter;
            *counter += 1;
            id
        };
        map.lock().unwrap().insert(id, tx.clone());

        let map_clone = Arc::clone(&map);
        println!("Client {} connected", id);

        tokio::spawn(handle_client(socket, rx, id, map_clone));
    }
}

async fn handle_client(
    mut socket: TcpStream,
    mut rx: mpsc::Receiver<String>,
    id: usize,
    map: Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>,
) {
    let mut buf = [0; 1024];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(n) if n == 0 => return, // socket closed
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                map.lock().unwrap().remove(&id);
                return;
            }
        };

        if let Err(e) = socket.write_all(&buf[0..n]).await {
            eprintln!("failed to write to socket; err = {:?}", e);
            map.lock().unwrap().remove(&id);
            return;
        }

        while let Some(message) = rx.recv().await {
            if let Err(e) = socket.write_all(message.as_bytes()).await {
                eprintln!("failed to write to socket; err = {:?}", e);
                map.lock().unwrap().remove(&id);
                return;
            }
        }
    }
}
