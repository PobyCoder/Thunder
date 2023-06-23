use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use log::{info};
use serde_json::{Value};
use crate::socket::TotalSocket;

pub async fn router(msg: Message, socket: SocketAddr) {
    let msg = msg.to_text().unwrap();
    let value: Value = serde_json::from_str(msg).unwrap();
    if let Some(data) = value.as_object().unwrap().iter().next() {
        let router = data.0;
        match router.as_str() {
            "login" => {
                let total_socket = TotalSocket::new(Mutex::new(HashMap::new()));
                let (tx, rx) = unbounded();
                total_socket.clone().lock().unwrap().insert(socket, tx);
                let peers = total_socket.lock().unwrap();
                let broadcast_recipients = peers.iter().filter(|(peer_addr, _)| peer_addr != &&socket).map(|(_, ws_sink)| ws_sink);
                info!("z");
                for recp in broadcast_recipients {
                    info!("ㅁㅁㅁㅁ");
                    recp.unbounded_send(msg.into()).unwrap();
                }
                info!("login : {}", data.0);
            }
            _ => {
                // // let total_socket = TotalSocket::new(Mutex::new(HashMap::new()));
                // let peers = total_socket.lock().unwrap();
                // // We want to broadcast the message to everyone except ourselves.
                // let broadcast_recipients = peers.iter().filter(|(peer_addr, _)| peer_addr != &&socket).map(|(_, ws_sink)| ws_sink);

                // for recp in broadcast_recipients {
                //     info!("asdasdas");
                //     recp.unbounded_send(msg.into()).unwrap();
                // }
                info!("default : {}", data.0);
            }
        }
    } else {
        
    }
}