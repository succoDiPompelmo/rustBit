mod buffer;
mod download;
pub mod manager;
pub mod piece_pool;
mod stream;

use std::collections::HashMap;

use log::info;

use crate::common::generator::generate_peer_id;
use crate::messages::{ContentType, Message};
use crate::peer::stream::{
    read_stream, send_metadata_handshake_request, write_stream, StreamInterface,
};

#[derive(Debug)]
pub struct Peer {
    id: String,
    choked: bool,
    active: bool,
    bitfield: Vec<bool>,
    metadata_size: usize,
    extensions: HashMap<String, u8>,
    stream: StreamInterface,
    info_hash: Vec<u8>,
}

impl Peer {
    pub fn new(stream: StreamInterface, info_hash: &[u8]) -> Peer {
        Peer {
            choked: true,
            bitfield: vec![],
            metadata_size: 0,
            extensions: HashMap::new(),
            stream,
            active: false,
            info_hash: info_hash.to_vec(),
            id: generate_peer_id(),
        }
    }

    pub fn is_choked(&self) -> bool {
        self.choked
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.to_vec()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_ready(&self) -> bool {
        self.metadata_size != 0 && !self.choked
    }

    pub fn get_peer_id(&self) -> String {
        self.id.to_owned()
    }

    #[cfg(test)]
    pub fn get_bitfield(&self) -> Vec<bool> {
        self.bitfield.to_vec()
    }

    pub fn get_metadata_size(&self) -> usize {
        self.metadata_size
    }

    pub fn get_extension_id_by_name(&self, name: &str) -> u8 {
        *self.extensions.get(name).unwrap()
    }

    fn apply_message(&mut self, message: &Message) {
        match message.get_id() {
            0 => {
                info!("Peer {:?} get a CHOKE message", self.get_peer_id());
                self.choked = true;
            }
            1 => {
                info!("Peer {:?} get a UNCHOKE message", self.get_peer_id());
                self.choked = false;
            }
            2 => {
                info!("Peer {:?} get a BITFIELD message", self.get_peer_id());
                self.apply_content(message);
            }
            5 => {
                info!("Peer {:?} get a ?? message", self.get_peer_id());
                self.apply_content(message)
            }
            19 => {
                info!("Peer {:?} get a HANDSHAKE message", self.get_peer_id());
                self.apply_content(message)
            }
            20 => {
                info!("Peer {:?} get a EXTENSION message", self.get_peer_id());
                self.apply_content(message)
            }
            _ => (),
        }
    }

    fn apply_content(&mut self, message: &Message) {
        match message.get_content() {
            ContentType::Bitfield(content) => self.bitfield = content.get_bitfield_as_bit_vector(),
            ContentType::Extension(content) => {
                if content.is_handshake() {
                    self.extensions = content.get_extensions().clone();
                    self.metadata_size = content.get_metadata_size().unwrap_or(0);
                }
            }
            ContentType::Handshake(handshake) => {
                self.active = self.info_hash == handshake.get_info_hash()
            }
            _ => info!(
                "Peer {:?} message content type not managed for: {:?}",
                self.get_peer_id(),
                message
            ),
        }
    }

    pub fn read_message(&mut self) -> Option<Message> {
        if let Some((body, id, length)) = read_stream(&mut self.stream) {
            return Message::new_raw(body, length, id).ok();
        }
        None
    }

    pub fn send_message(&mut self, message: Message) {
        write_stream(&mut self.stream, &message.as_bytes())
    }

    pub fn send_metadata_handshake_request(&mut self) -> Result<(), &'static str> {
        send_metadata_handshake_request(&mut self.stream)
    }

    #[cfg(test)]
    pub fn set_metadata_size(&mut self, metadata_size: usize) {
        self.metadata_size = metadata_size;
    }

    #[cfg(test)]
    pub fn add_extension(&mut self, extension_key: String, extension_value: u8) {
        self.extensions.insert(extension_key, extension_value);
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, vec};

    use crate::{
        messages::{new_bitfield, new_extension, new_generic_message},
        peer::stream::StreamInterface,
    };

    use super::Peer;

    #[test]
    fn test_apply_choke_messages() {
        let mut peer = Peer::new(StreamInterface::Nothing(), &[]);

        let choke_message = new_generic_message(0, 5);
        peer.apply_message(&choke_message);
        assert!(peer.is_choked());

        let unchoke_message = new_generic_message(1, 5);
        peer.apply_message(&unchoke_message);
        assert!(!peer.is_choked());
    }

    #[test]
    fn test_apply_bitfield_message() {
        let mut peer = Peer::new(StreamInterface::Nothing(), &[]);

        let bitfield = vec![0x10];
        let bitfield_message = new_bitfield(bitfield);
        peer.apply_message(&bitfield_message);
        assert_eq!(
            peer.get_bitfield(),
            vec![false, false, false, true, false, false, false, false]
        )
    }

    #[test]
    fn test_apply_extension_message() {
        let mut peer = Peer::new(StreamInterface::Nothing(), &[]);

        let extension_message =
            new_extension(10, HashMap::from([("ut_metadata".to_owned(), 2)]), 123);
        peer.apply_message(&extension_message);
        assert_eq!(peer.get_extension_id_by_name("ut_metadata"), 2);
        assert_eq!(peer.get_metadata_size(), 123);
    }
}
