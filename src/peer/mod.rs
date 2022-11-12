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
        Peer {
            choked: true,
            bitfield: vec![],
            metadata_size: 0,
            extensions: HashMap::new(),
            stream: PeerStream::new(stream),
        }
    }

    pub fn is_choked(&self) -> bool {
        self.choked
    }

    pub fn send_message(&mut self, message: Message) {
        self.stream.write_stream(&message.as_bytes());
    }

    pub fn get_metadata_size(&self) -> usize {
        self.metadata_size
    }

    pub fn get_extension_id_by_name(&self, name: &str) -> u8 {
        *self.extensions.get(name).unwrap()
    }

    pub fn read_message(&mut self) -> Option<Message> {
        self.stream
            .read_stream()
            .map(|(body, id, length)| Message::new_raw(body, length, id))
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
                if content.is_handshake() {
                    self.extensions = content.get_extensions().clone();
                    self.metadata_size = content.get_metadata_size().unwrap_or(0);
                }
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {}
