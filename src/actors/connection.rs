use actix::prelude::*;

use crate::{
    actors::messages::{PieceDownloadFailed, PieceDownloadSuccessfull},
    peer::manager::download,
};

use super::messages::PieceRequested;

pub struct ConnectionActor;

// Provide Actor implementation for our actor
impl Actor for ConnectionActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, ctx: &mut SyncContext<Self>) {
        println!("Torrent Actor is alive");
    }

    fn stopped(&mut self, ctx: &mut SyncContext<Self>) {
        println!("Torrent Actor is stopped");
    }
}

impl Handler<PieceRequested> for ConnectionActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceRequested, ctx: &mut Self::Context) -> Self::Result {
        println!("Start the download of piece {:?}", msg.piece_idx);

        match download(msg.endpoint.clone(), &msg.info, msg.piece_idx) {
            Ok(piece) => {
                println!("OK");
                let _ = msg.torrent_actor.do_send(PieceDownloadSuccessfull {
                    endpoint: msg.endpoint.clone(),
                    piece,
                    piece_idx: msg.piece_idx,
                });
            }
            Err(err) => {
                println!("KO");
                let _ = msg.torrent_actor.do_send(PieceDownloadFailed {
                    endpoint: msg.endpoint,
                    piece_idx: msg.piece_idx,
                });
            }
        }

        Ok(true)
    }
}
