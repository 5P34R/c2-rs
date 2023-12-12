use tokio::{net::TcpStream, io::{AsyncReadExt}};

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => println!("Client finished successfully"),
        Err(e) => eprintln!("Client error: {:?}", e),
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect("localhost:9999").await?;
    println!("Connected to server");
    loop {
        let mut buf = [0; 1024];
        match socket.read(&mut buf).await {
            Ok(n) => println!("Received: {}", String::from_utf8_lossy(&buf[..n])),
            Err(e) => eprintln!("Read error: {:?}", e),
        }
    }
}