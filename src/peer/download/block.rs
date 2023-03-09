use std::cmp;

use crate::{
    messages::{new_request, Message},
    peer::Peer,
};

pub fn next_block(
    block_size: usize,
    piece_index: usize,
    total_length: usize,
    piece_length: usize,
) -> impl FnMut(&mut Peer, usize) {
    move |peer: &mut Peer, block_index: usize| {
        let block_offset = block_index * block_size;
        let block_length = get_block_size(
            block_offset,
            total_length,
            piece_index * piece_length,
            block_size,
        );
        peer.send_message(new_request(
            piece_index as u32,
            block_offset as u32,
            block_length as u32,
        ));
    }
}

pub fn message_filter() -> fn(&Message) -> bool {
    Message::is_request_message
}

fn get_block_size(
    block_offset: usize,
    torrent_length: usize,
    piece_offset: usize,
    block_size: usize,
) -> usize {
    cmp::min(torrent_length - (block_offset + piece_offset), block_size)
}