use hyperion_core::block::{Block, Serializable};
use hyperion_core::crypto::Hashable;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncReadExt;

pub async fn start_network_listener(addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_client(socket));
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let block: Block = Block::from_bytes(&buffer[..n]).unwrap();
    println!("Received block: {:?}", block.double_sha256());
}