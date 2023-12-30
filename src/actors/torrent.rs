use std::{fs::File, io::Read};

use log::info;

use actix::prelude::*;

use crate::{
    actors::messages::PieceReady,
    peer::{manager::get_info, piece_pool::PiecePool},
    torrent::info::Info,
};

use super::{
    connection::ConnectionActor,
    messages::{PeerFound, PieceDownloadFailed, PieceDownloadSuccessfull, PieceRequested},
    writer::WriterActor,
};

use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct TorrentActor {
    connections_pool: Addr<ConnectionActor>,
    pub info: Option<Info>,
    pub info_hash: Vec<u8>,
    peers: Vec<Peer>,
    piece_available_pool: Option<PiecePool>,
    writers_pool: Addr<WriterActor>,
    initiated: bool,
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
            writers_pool: write_addr,
            initiated: false,
        }
    }
}

// Provide Actor implementation for our actor
impl Actor for TorrentActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
    }
}

impl Handler<PieceDownloadSuccessfull> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceDownloadSuccessfull, ctx: &mut Context<Self>) -> Self::Result {
        let msg_ready = PieceReady {
            piece: msg.piece,
            files: self.info.as_ref().unwrap().get_files().unwrap(),
            piece_idx: msg.piece_idx,
            piece_length: self.info.as_ref().unwrap().get_piece_length(),
            torrent_actor: ctx.address(),
        };
        self.writers_pool.do_send(msg_ready);

        let endpoint = msg.endpoint;
        Peer::update_sucess(&mut self.peers, endpoint.clone());

        if let Some(piece_idx) = self.piece_available_pool.as_mut().unwrap().pop() {
            let msg = PieceRequested {
                piece_idx,
                info: self.info.as_ref().unwrap().clone(),
                endpoint,
                torrent_actor: ctx.address(),
            };
            self.connections_pool.do_send(msg);
        }

        Ok(true)
    }
}

impl Handler<PieceDownloadFailed> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceDownloadFailed, ctx: &mut Context<Self>) -> Self::Result {
        self.piece_available_pool
            .as_ref()
            .unwrap()
            .insert(msg.piece_idx);

        let endpoint = msg.endpoint.as_str();
        Peer::update_failed(&mut self.peers, endpoint.to_string());

        let endpoint = Peer::find_suitable_peer(self.peers.to_vec());

        if let Some(piece_idx) = self.piece_available_pool.as_mut().unwrap().pop() {
            let msg = PieceRequested {
                piece_idx,
                info: self.info.as_ref().unwrap().clone(),
                endpoint,
                torrent_actor: ctx.address(),
            };
            self.connections_pool.do_send(msg);
        }

        Ok(true)
    }
}

impl Handler<PeerFound> for TorrentActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PeerFound, ctx: &mut Context<Self>) -> Self::Result {
        self.peers.push(Peer::new(msg.peer.endpoint()));

        match &self.info {
            None => {
                let filename = urlencoding::encode_binary(&self.info_hash).into_owned();
                let file_path = format!("./downloads/{filename}");
                std::fs::create_dir_all("./downloads").unwrap();

                if let Ok(mut info_file) = File::open(&file_path) {
                    info!("Torrent info from file: {:?}", filename);
                    let mut info_buffer = "".to_string();
                    info_file.read_to_string(&mut info_buffer).unwrap();

                    let info: Info = serde_json::from_str(&info_buffer).unwrap();

                    let piece_count = (0..info.get_total_length())
                        .step_by(info.get_piece_length())
                        .len();

                    self.piece_available_pool = Some(PiecePool::new(piece_count));
                    self.info = Some(info);

                    return Ok(true);
                }

                if let Ok(info) = get_info(&self.info_hash, msg.peer.endpoint()) {
                    let piece_count = (0..info.get_total_length())
                        .step_by(info.get_piece_length())
                        .len();

                    self.piece_available_pool = Some(PiecePool::new(piece_count));

                    serde_json::to_writer(&File::create(&file_path).unwrap(), &info).unwrap();
                    self.info = Some(info);

                    return Ok(true);
                };
            }
            Some(info) => {
                if self.peers.len() > 10 && !self.initiated {
                    for _ in 0..5 {
                        let endpoint = Peer::find_suitable_peer(self.peers.to_vec());

                        if let Some(piece_idx) = self.piece_available_pool.as_mut().unwrap().pop() {
                            let msg = PieceRequested {
                                piece_idx,
                                info: info.clone(),
                                endpoint,
                                torrent_actor: ctx.address(),
                            };

                            self.connections_pool.do_send(msg);
                            self.initiated = true;
                        }
                    }
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, Debug)]
struct Peer {
    endpoint: String,
    piece_downloaded: usize,
    piece_failed: usize,
}

impl Peer {
    fn new(endpoint: String) -> Peer {
        Peer {
            endpoint,
            piece_downloaded: 0,
            piece_failed: 0,
        }
    }

    fn update_failed(pool: &mut Vec<Peer>, endpoint: String) {
        for peer in pool {
            if peer.endpoint == endpoint {
                peer.piece_failed += 1;
            }
        }
    }

    fn update_sucess(pool: &mut Vec<Peer>, endpoint: String) {
        for peer in pool {
            if peer.endpoint == endpoint {
                peer.piece_downloaded += 1;
            }
        }
    }

    fn find_suitable_peer(mut pool: Vec<Peer>) -> String {
        pool.shuffle(&mut thread_rng());

        for peer in &pool {
            if peer.piece_failed < 2 {
                return peer.endpoint.to_owned();
            }
        }

        for peer in &pool {
            if peer.piece_failed < 4 {
                return peer.endpoint.to_owned();
            }
        }

        return pool.first().unwrap().endpoint.to_owned();
    }
}
