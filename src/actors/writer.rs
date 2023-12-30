use actix::prelude::*;

use crate::torrent::writer::write;

use super::messages::PieceReady;

pub struct WriterActor;

// Provide Actor implementation for our actor
impl Actor for WriterActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut SyncContext<Self>) {}

    fn stopped(&mut self, _ctx: &mut SyncContext<Self>) {}
}

impl Handler<PieceReady> for WriterActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: PieceReady, _ctx: &mut Self::Context) -> Self::Result {
        let _ = write(msg.piece, msg.piece_idx, msg.files, msg.piece_length);

        Ok(true)
    }
}
