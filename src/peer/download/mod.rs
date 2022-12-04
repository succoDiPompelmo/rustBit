pub mod block;
pub mod info;

use crate::peer::buffer::MessageBuffer;
use crate::peer::Peer;

pub const BLOCK_SIZE: usize = 16384;
pub const INFO_PIECE_SIZE: usize = 16384;

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
                (0..piece_length).step_by(BLOCK_SIZE).len(),
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

#[cfg(test)]
mod test {
    use crate::{peer::stream::StreamInterface, common::mock_stream::MockStream};

    use super::*;

    #[test]
    fn test_download_info() {

        let mut s = MockStream::new();
        // UNCHOKE MESSAGE
        s.push_bytes_to_read([0, 0, 0, 1, 1].as_slice());
        // EXTENSION DATA MESSAGE
        let dictionary = "d8:msg_typei1ee".as_bytes().to_vec();
        let message = [vec![0, 0, 0, 22, 20, 2], dictionary, vec![1, 2, 3, 4, 5]].concat();
        s.push_bytes_to_read(&message);
    
        let e = StreamInterface::Mocked(s.clone());

        let mut peer = Peer::new(e, &[]);

        peer.set_metadata_size(INFO_PIECE_SIZE);
        peer.add_extension("ut_metadata".to_owned(), 1);

        let downloadable = Downloadable::Info;
        assert_eq!(download(&mut peer, downloadable), Ok(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_download_piece() {

        let mut s = MockStream::new();
        // UNCHOKE MESSAGE
        s.push_bytes_to_read([0, 0, 0, 1, 1].as_slice());
        // PIECE MESSAGE
        let message = vec![0, 0, 0, 14, 6, 0, 0, 0, 0, 0, 0, 0, 0, 5, 4, 3, 2, 1];
        s.push_bytes_to_read(&message);
    
        let e = StreamInterface::Mocked(s.clone());

        let mut peer = Peer::new(e, &[]);

        let downloadable = Downloadable::Block((16384, 0, 16384));
        assert_eq!(download(&mut peer, downloadable), Ok(vec![5, 4, 3, 2, 1]));
    }
}
