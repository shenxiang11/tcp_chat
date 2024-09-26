use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use log::{info, warn};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use crate::message::Message;

#[derive(Default)]
pub(crate) struct State {
    // peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>
    peers: DashMap<SocketAddr,  SplitSink<Framed<TcpStream, LinesCodec>, String>>
}

pub(crate) struct Peer {
    pub(crate) username: String,
    pub(crate) stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

// const MAX_MESSAGES: usize = 128;

impl State {
    pub(crate) async fn insert_peer(&self, addr: SocketAddr, username: String, stream: Framed<TcpStream, LinesCodec>) -> Peer {
        // let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        // self.peers.insert(addr, tx);

        let (stream_sender, stream_receiver) = stream.split();
        self.peers.insert(addr, stream_sender);

        // tokio::spawn(async move {
        //     while let Some(message) = rx.recv().await {
        //         if let Err(e) = stream_sender.send(message.to_string()).await {
        //             warn!("Failed to send message to {}: {}", addr, e);
        //             break;
        //         }
        //     }
        // });

        Peer {
            username,
            stream: stream_receiver,
        }
    }

    pub(crate) fn remove_peer(&self, addr: &SocketAddr) {
        self.peers.remove(addr);
    }

    pub(crate) async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for mut peer in self.peers.iter_mut() {
            if peer.key() != &addr {
                let mut s = peer.value_mut();
                if let Err(e) = s.send(message.to_string()).await {
                    warn!("Failed to broadcast message to {}: {}", peer.key(), e);
                    self.remove_peer(peer.key());
                }
            }
        }
    }
}
