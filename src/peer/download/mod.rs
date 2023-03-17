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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum DownloadableError {
    #[error("Choked Peer")]
    ChokedPeer(),
    #[error("Download idle for too long")]
    Idle(),
}

impl Downloadable {
    pub fn download(&self, peer: &mut Peer) -> Result<Vec<u8>, DownloadableError> {
        match self {
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
                    block::next_block(BLOCK_SIZE, *piece_index, *total_length, *piece_length),
                    (0..real_piece_length(*piece_length, *piece_index, *total_length))
                        .step_by(BLOCK_SIZE)
                        .len(),
                );

                execute(peer, buffer)
            }
        }
    }
}

fn real_piece_length(piece_length: usize, piece_index: usize, total_length: usize) -> usize {
    if piece_length * (piece_index + 1) > total_length {
        total_length - piece_length * piece_index
    } else {
        piece_length
    }
}

fn execute<F>(peer: &mut Peer, mut buffer: MessageBuffer<F>) -> Result<Vec<u8>, DownloadableError>
where
    F: FnMut(&mut Peer, usize),
{
    let mut idle_count = 0;

    loop {
        peer.read_message().map_or((), |msg| {
            peer.apply_message(&msg);
            buffer.push_message(msg);
            idle_count = 0;
        });

        if buffer.is_full() {
            return Ok(buffer.assemble_content());
        };

        if peer.is_choked() {
            return Err(DownloadableError::ChokedPeer());
        }

        buffer.request_next_message(peer);

        idle_count += 1;

        if idle_count > 10 {
            return Err(DownloadableError::Idle());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{common::mock_stream::MockStream, peer::stream::StreamInterface};

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
        assert_eq!(downloadable.download(&mut peer), Ok(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_download_piece() {
        let mut s = MockStream::new();
        // UNCHOKE MESSAGE
        s.push_bytes_to_read([0, 0, 0, 1, 1].as_slice());
        // PIECE MESSAGE
        let body = vec![2; 12384];
        let length = (12384 + 9) as u32;
        let message = [
            length.to_be_bytes().to_vec(),
            vec![6, 0, 0, 0, 0, 0, 0, 0, 0],
            body.to_vec(),
        ]
        .concat();
        s.push_bytes_to_read(&message);

        let e = StreamInterface::Mocked(s.clone());

        let mut peer = Peer::new(e, &[]);

        let downloadable = Downloadable::Block((16384 * 2, 1, 16384 * 2 + 12384));
        assert_eq!(downloadable.download(&mut peer), Ok(body));
    }
}
