use std::collections::HashMap;
use std::sync::Mutex;

use larptrix_protocol::ServerMessage;
use tokio::sync::mpsc;
use uuid::Uuid;

pub type Outbound = mpsc::UnboundedSender<ServerMessage>;

struct Peer {
    display_name: String,
    conns: Vec<Outbound>,
}

pub struct Hub {
    peers: Mutex<HashMap<Uuid, Peer>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            peers: Mutex::new(HashMap::new()),
        }
    }

    pub fn join(&self, user_id: Uuid, display_name: String, tx: Outbound) {
        let mut peers = self.peers.lock().expect("hub lock");
        let peer = peers.entry(user_id).or_insert(Peer {
            display_name: display_name.clone(),
            conns: Vec::new(),
        });
        peer.display_name = display_name;
        peer.conns.push(tx);
    }

    pub fn leave(&self, user_id: Uuid, tx: &Outbound) {
        let mut peers = self.peers.lock().expect("hub lock");
        if let Some(peer) = peers.get_mut(&user_id) {
            peer.conns.retain(|conn| !conn.same_channel(tx));
            if peer.conns.is_empty() {
                peers.remove(&user_id);
            }
        }
    }

    pub fn online_ids(&self) -> Vec<String> {
        let peers = self.peers.lock().expect("hub lock");
        peers.keys().map(ToString::to_string).collect()
    }

    pub fn broadcast(&self, message: ServerMessage) {
        let peers = self.peers.lock().expect("hub lock");
        for peer in peers.values() {
            for conn in &peer.conns {
                let _ = conn.send(message.clone());
            }
        }
    }

    pub fn send_to(&self, user_id: Uuid, message: ServerMessage) -> usize {
        let peers = self.peers.lock().expect("hub lock");
        if let Some(peer) = peers.get(&user_id) {
            let mut delivered = 0;
            for conn in &peer.conns {
                if conn.send(message.clone()).is_ok() {
                    delivered += 1;
                }
            }
            delivered
        } else {
            0
        }
    }
}
