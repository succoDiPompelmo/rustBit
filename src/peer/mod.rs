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

    pub fn apply_message(&mut self, message: &Message) {
        match message.get_id() {
            0 => {
                println!("CHOKE MESSAGE");
                self.choke();
            }
            1 => {
                println!("UNCHOKE MESSAGE");
                self.unchoke();
            }
            5 => {
                println!("BITFIELD MESSAGE");
                if let ContentType::Interested(interested_message) = message.get_content() {
                    self.set_bitfield(interested_message.get_bitfield_as_bit_vector());
                }
            }
            20 => {
                println!("EXTENSION MESSAGE");
                if let ContentType::Extension(extension_message) = message.get_content() {
                    if !extension_message.is_handshake() {
                        return;
                    }

                    self.extensions = extension_message.get_extensions().clone();
                    self.metadata_size = extension_message.get_metadata_size().unwrap_or(0);
                }
            }
            _ => (),
        }
    }

    pub fn unchoke(&mut self) {
        self.choked = false;
    }

    pub fn choke(&mut self) {
        self.choked = true;
    }

    pub fn is_choked(&self) -> bool {
        return self.choked;
    }

    pub fn set_bitfield(&mut self, bitfield: Vec<bool>) {
        self.bitfield = bitfield;
    }

    pub fn is_piece_available(&self, piece_index: usize) -> bool {
        self.bitfield[piece_index]
    }

    pub fn get_stream(&mut self) -> &mut PeerStream {
        &mut self.stream
    }

    pub fn get_extension_id(&self, key: &str) -> Option<u8> {
        match self.extensions.get(key) {
            Some(value) => Some(*value),
            None => None,
        }
    }

    pub fn set_extension(&mut self, extensions: HashMap<String, u8>) {
        self.extensions = extensions;
    }

    pub fn get_metadata_size(&self) -> usize {
        self.metadata_size
    }

    pub fn set_metadata_size(&mut self, metadata_size: usize) {
        self.metadata_size = metadata_size;
    }
}

#[cfg(test)]
mod test {}
