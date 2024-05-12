use std::net::SocketAddr;

use anyhow::Result;
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    //build a listener
    let addr = "0.0.0.0:6379";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);
    loop {
        let (socket, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);

        //spawn a new task for each incoming connection
        tokio::spawn(async move {
            if let Err(e) = process_redis(socket, raddr).await {
                warn!("Error processing connection from {}: {:?}", raddr, e);
            }
        });
    }
}

async fn process_redis(socket: tokio::net::TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        socket.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match socket.try_read_buf(&mut buf) {
            Ok(0) => {
                info!("Connection closed");
                break;
            }
            Ok(n) => {
                info!("Received {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("Received: {}", line);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                info!("Would block");
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Connection closed by peer: {}", raddr);

    Ok(())
}
