use crate::{
    messages::{new_metadata, Message},
    peer::Peer,
};

pub fn next_piece() -> impl FnMut(&mut Peer, usize) {
    |peer: &mut Peer, piece_index| {
        let metadata_id = peer.get_extension_id_by_name("ut_metadata");
        peer.send_message(new_metadata(metadata_id, piece_index))
    }
}

pub fn message_filter() -> fn(&Message) -> bool {
    Message::is_extension_data_message
}
