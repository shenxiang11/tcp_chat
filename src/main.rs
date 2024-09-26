mod state;
mod message;

use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use futures::SinkExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};
use crate::message::Message;
use crate::state::State;

const LISTEN_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {}", LISTEN_ADDR);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);

        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(state, stream, addr).await {
                warn!("Failed to handle connection {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(state: Arc<State>, stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Welcome to the chat server!\nEnter your name: ").await?;

    let username = match stream.next().await {
        Some(Ok(name)) => name,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.insert_peer(addr, username.clone(), stream).await;

    let message = Arc::new(Message::user_join(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message.clone()).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => return Err(e.into()),
        };

        let message = Arc::new(Message::chat(&peer.username, line.clone()));
        state.broadcast(addr, message.clone()).await;
    }

    state.remove_peer(&addr);

    let message = Arc::new(Message::user_leave(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message.clone()).await;

    Ok(())
}
