pub mod handshake;
pub mod peer_manager;
pub mod peer_stream;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::messages::{ContentType, Message};
use crate::peer::handshake::Handshake;
use crate::peer::peer_stream::PeerStream;

#[derive(Debug)]
pub struct Peer {
    choked: bool,
    bitfield: Vec<bool>,
    metadata_size: usize,
    extensions: HashMap<String, u8>,
    stream: Option<PeerStream>,
}

impl Peer {
    pub fn new(stream: Option<TcpStream>) -> Peer {
        Peer {
            choked: true,
            bitfield: vec![],
            metadata_size: 0,
            extensions: HashMap::new(),
            stream: stream.map(PeerStream::new),
        }
    }

    pub fn connect(endpoint: String) -> Result<Peer, &'static str> {
        let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");

        let connect_timeout = Duration::from_secs(3);

        match TcpStream::connect_timeout(&server, connect_timeout) {
            Ok(stream) => Ok(Peer::new(Some(stream))),
            Err(_) => Err("Ciao"),
        }
    }

    pub fn handshake(&mut self, info_hash: &[u8], peer_id: &str) -> Result<(), &'static str> {
        match &mut self.stream {
            Some(stream) => {
                let handshake_request = Handshake::new(info_hash, peer_id);
                stream.write_stream(&handshake_request.as_bytes());

                match stream.read_handshake() {
                    Ok(buffer) => {
                        println!("{:?}", buffer);
                        let hadnshake_response = Handshake::from_bytes(buffer);
                        if hadnshake_response.get_info_hash() != *info_hash {
                            return Err("Info hash not matching in handshake response");
                        }
                    }
                    Err(_) => return Err("Reading handhsake response has failed"),
                }

                Ok(())
            }
            None => Err("No stream available"),
        }
    }

    pub fn is_choked(&self) -> bool {
        self.choked
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
                println!("CHOKE MESSAGE");
                self.choked = true;
            }
            1 => {
                println!("UNCHOKE MESSAGE");
                self.choked = false;
            }
            2 => {
                println!("BITFIELD MESSAGE");
                self.apply_content(message);
            }
            5 => {
                println!("INTERESTED MESSAGE");
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
            ContentType::Bitfield(content) => self.bitfield = content.get_bitfield_as_bit_vector(),
            ContentType::Extension(content) => {
                if content.is_handshake() {
                    self.extensions = content.get_extensions().clone();
                    self.metadata_size = content.get_metadata_size().unwrap_or(0);
                }
            }
            _ => (),
        }
    }

    pub fn read_message(&mut self) -> Option<Message> {
        match &mut self.stream {
            Some(stream) => stream
                .read_stream()
                .map(|(body, id, length)| Message::new_raw(body, length, id)),
            None => None,
        }
    }

    pub fn send_message(&mut self, message: Message) {
        match &mut self.stream {
            Some(stream) => stream.write_stream(&message.as_bytes()),
            None => (),
        }
    }

    pub fn send_metadata_handshake_request(&mut self) {
        match &mut self.stream {
            Some(stream) => stream.send_metadata_handshake_request(),
            _ => (),
        };
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, vec};

    use crate::messages::{new_bitfield, new_extension, new_generic_message};

    use super::Peer;

    #[test]
    fn test_apply_choke_messages() {
        let mut peer = Peer::new(None);

        let choke_message = new_generic_message(0, 5);
        peer.apply_message(&choke_message);
        assert!(peer.is_choked());

        let unchoke_message = new_generic_message(1, 5);
        peer.apply_message(&unchoke_message);
        assert!(!peer.is_choked());
    }

    #[test]
    fn test_apply_bitfield_message() {
        let mut peer = Peer::new(None);

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
        let mut peer = Peer::new(None);

        let extension_message =
            new_extension(10, HashMap::from([("ut_metadata".to_owned(), 2)]), 123);
        peer.apply_message(&extension_message);
        assert_eq!(peer.get_extension_id_by_name("ut_metadata"), 2);
        assert_eq!(peer.get_metadata_size(), 123);
    }
}
