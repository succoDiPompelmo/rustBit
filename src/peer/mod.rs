pub mod peer_manager;
pub mod peer_stream;

use std::collections::HashMap;
use std::net::TcpStream;

use crate::messages::{ContentType, Message};
use crate::peer::peer_stream::PeerStream;

#[derive(Debug)]
pub struct Peer {
    choked: bool,
    bitfield: Vec<bool>,
    metadata_size: usize,
    extensions: HashMap<String, u8>,
    stream: PeerStream,
}

impl Peer {
    pub fn new(stream: TcpStream) -> Peer {
        return Peer {
            choked: true,
            bitfield: vec![],
            metadata_size: 0,
            extensions: HashMap::new(),
            stream: PeerStream::new(stream),
        };
    }

    pub fn is_choked(&self) -> bool {
        return self.choked;
    }

    pub fn get_stream(&mut self) -> &mut PeerStream {
        &mut self.stream
    }

    pub fn get_metadata_size(&self) -> usize {
        self.metadata_size
    }

    pub fn request_info_piece(&mut self, offset: usize) {
        self.stream
            .send_metadata_request(*self.extensions.get("ut_metadata").unwrap(), offset);
    }

    pub fn request_piece(&mut self, block_size: u32, block_offset: u32, piece_idx: u32) {
        self.stream
            .send_request(block_size, block_offset, piece_idx);
    }

    fn apply_message(&mut self, message: &Message) {
        match message.get_id() {
            0 => {
                println!("CHOKE MESSAGE");
                self.choked = true;
            }
            1 => {
                println!("UNCHOKE MESSAGE");
                self.choked = false;
            }
            5 => {
                println!("BITFIELD MESSAGE");
                self.apply_content(message)
            }
            20 => {
                println!("EXTENSION MESSAGE");
                self.apply_content(message)
            }
            _ => (),
        }
    }

    fn apply_content(&mut self, message: &Message) {
        match message.get_content() {
            ContentType::Interested(content) => {
                self.bitfield = content.get_bitfield_as_bit_vector()
            }
            ContentType::Extension(content) => {
                if !content.is_handshake() {
                    return;
                }

                self.extensions = content.get_extensions().clone();
                self.metadata_size = content.get_metadata_size().unwrap_or(0);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {}
