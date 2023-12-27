use actix::prelude::*;

use crate::{
    torrent::{file::File, info::Info},
    tracker::peer_endpoint::PeerEndpoint,
};

use super::torrent::TorrentActor;

// EVENTS

#[derive(Clone, Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct TorrentRegistered {
    pub info_hash: Vec<u8>,
    pub torrent_actor_addr: Addr<TorrentActor>,
}

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct PeerFound {
    pub peer: PeerEndpoint,
}

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct PieceDownloadSuccessfull {
    pub endpoint: String,
    pub piece: Vec<u8>,
    pub piece_idx: usize,
}

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct PieceDownloadFailed {
    pub endpoint: String,
    pub piece_idx: usize,
}

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct PieceReady {
    pub piece: Vec<u8>,
    pub files: Vec<File>,
    pub piece_idx: usize,
    pub piece_length: usize,
    pub torrent_actor: Addr<TorrentActor>,
}

// COMMANDS

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct PieceRequested {
    pub endpoint: String,
    pub info: Info,
    pub piece_idx: usize,
    pub torrent_actor: Addr<TorrentActor>,
}
