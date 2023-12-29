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

    fn started(&mut self, _ctx: &mut SyncContext<Self>) {}

    fn stopped(&mut self, _ctx: &mut SyncContext<Self>) {}
}

impl Handler<PieceRequested> for ConnectionActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceRequested, _ctx: &mut Self::Context) -> Self::Result {
        match download(msg.endpoint.clone(), &msg.info, msg.piece_idx) {
            Ok(piece) => {
                msg.torrent_actor.do_send(PieceDownloadSuccessfull {
                    endpoint: msg.endpoint.clone(),
                    piece,
                    piece_idx: msg.piece_idx,
                });
            }
            Err(_) => {
                msg.torrent_actor.do_send(PieceDownloadFailed {
                    endpoint: msg.endpoint,
                    piece_idx: msg.piece_idx,
                });
            }
        }

        Ok(true)
    }
}
