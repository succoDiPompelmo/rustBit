pub mod block;
pub mod info;

use crate::peer::buffer::MessageBuffer;
use crate::peer::Peer;

const BLOCK_SIZE: usize = 16384;
const INFO_PIECE_SIZE: usize = 16384;

pub enum Downloadable {
    Info,
    Block((usize, usize, usize)),
}

pub fn download(peer: &mut Peer, download: Downloadable) -> Result<Vec<u8>, &'static str> {
    match download {
        Downloadable::Info => {
            let buffer = MessageBuffer::new(
                info::message_filter(),
                info::next_piece(),
                (0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len(),
            );

            execute(peer, buffer)
        }
        Downloadable::Block((piece_length, piece_index, total_length)) => {
            let buffer = MessageBuffer::new(
                block::message_filter(),
                block::next_block(BLOCK_SIZE, piece_index, total_length, piece_length),
                (0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len(),
            );

            execute(peer, buffer)
        }
    }
}

fn execute<F>(peer: &mut Peer, mut buffer: MessageBuffer<F>) -> Result<Vec<u8>, &'static str>
where
    F: FnMut(&mut Peer, usize),
{
    loop {
        peer.read_message().map_or((), |msg| {
            peer.apply_message(&msg);
            buffer.push_message(msg);
        });

        if buffer.is_full() {
            return Ok(buffer.assemble_content());
        };

        if peer.is_choked() {
            panic!("Chocked peer")
        }

        buffer.request_next_message(peer);
    }
}
