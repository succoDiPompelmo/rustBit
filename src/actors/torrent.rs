use actix::prelude::*;

use crate::{
    peer::{
        manager::{download, get_info},
        piece_pool::PiecePool,
    },
    torrent::info::Info,
    tracker::peer_endpoint::PeerEndpoint, actors::messages::PieceReady,
};

use super::{messages::{PeerFound, PieceRequested, PieceDownloadSuccessfull, PieceDownloadFailed}, connection::ConnectionActor, writer::WriterActor};

pub struct TorrentActor {
    connections_pool: Addr<ConnectionActor>,
    pub info: Option<Info>,
    pub info_hash: Vec<u8>,
    pub peers: Vec<PeerEndpoint>,
    piece_available_pool: Option<PiecePool>,
    writers_pool: Addr<WriterActor>
}

impl TorrentActor {
    pub fn new(info_hash: Vec<u8>) -> TorrentActor {

        let addr = SyncArbiter::start(5, || ConnectionActor);
        let write_addr = SyncArbiter::start(1, || WriterActor);

        TorrentActor {
            connections_pool: addr,
            info: None,
            info_hash,
            peers: vec![],
            piece_available_pool: None,
            writers_pool: write_addr
        }
    }
}

// Provide Actor implementation for our actor
impl Actor for TorrentActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Torrent Actor is alive");
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("Torrent Actor is stopped");
    }
}

impl Handler<PieceDownloadSuccessfull> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceDownloadSuccessfull, ctx: &mut Context<Self>) -> Self::Result {
        println!("Ok piece {:?}", msg.piece_idx);

        let msg_ready = PieceReady {piece: msg.piece, files: self.info.as_ref().unwrap().get_files().unwrap(), piece_idx: msg.piece_idx, piece_length: self.info.as_ref().unwrap().get_piece_length(), torrent_actor: ctx.address()};
        self.writers_pool.do_send(msg_ready);

        if let Some(piece_idx) =self.piece_available_pool.as_mut().unwrap().pop() {
            let msg = PieceRequested {piece_idx, info: self.info.as_ref().unwrap().clone(), endpoint: msg.endpoint, torrent_actor: ctx.address()};
            self.connections_pool.do_send(msg);
        }

        Ok(true)
    }
}

impl Handler<PieceDownloadFailed> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceDownloadFailed, ctx: &mut Context<Self>) -> Self::Result {
        self.piece_available_pool.as_ref().unwrap().insert(msg.piece_idx);

        println!("Failed piece {:?}", msg.piece_idx);
        
        let endpoint = msg.endpoint.as_str();

        for peer in &self.peers {
            if peer.endpoint() != endpoint {
                if let Some(piece_idx) = self.piece_available_pool.as_mut().unwrap().pop() {
                    let msg = PieceRequested {piece_idx, info: self.info.as_ref().unwrap().clone(), endpoint: endpoint.to_string(), torrent_actor: ctx.address()};
                    self.connections_pool.do_send(msg);
                }

                break;
            }
        }

        Ok(true)
    }
}

impl Handler<PeerFound> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PeerFound, ctx: &mut Context<Self>) -> Self::Result {        
        match &self.info {
            None => {
                if let Ok(info) = get_info(&self.info_hash, msg.peer.endpoint()) {
                    let piece_count = (0..info.get_total_length())
                        .step_by(info.get_piece_length())
                        .len();

                    self.piece_available_pool = Some(PiecePool::new(piece_count));

                    self.info = Some(info);
                };
            }
            Some(info) => {
                if let Some(piece_idx) =self.piece_available_pool.as_mut().unwrap().pop() {

                    let msg = PieceRequested {piece_idx, info: info.clone(), endpoint: msg.peer.endpoint(), torrent_actor: ctx.address()};

                    self.connections_pool.do_send(msg);
                }

                self.peers.push(msg.peer);
            }
        }

        Ok(true)
    }
}
