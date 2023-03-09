pub mod bitfield;
pub mod extension;
pub mod handshake;
pub mod interested;
pub mod request;

#[cfg(test)]
use std::collections::HashMap;

use crate::messages::bitfield::BitfieldMessage;
use crate::messages::extension::ExtensionMessage;
use crate::messages::handshake::HandshakeMessage;
use crate::messages::interested::InterestedMessage;
use crate::messages::request::RequestMessage;

#[derive(Debug, Clone)]
pub enum ContentType {
    Request(RequestMessage),
    Extension(ExtensionMessage),
    Interested(InterestedMessage),
    Bitfield(BitfieldMessage),
    Handshake(HandshakeMessage),
    Nothing(),
}

#[derive(Debug, Clone)]
pub struct Message {
    id: u8,
    length: u32,
    content: ContentType,
}

impl Message {
    // TODO: Maybe we could promote body to use a ContentType as parameter type, even though this function is used
    // during the read of a message.
    pub fn new_raw(body: Vec<u8>, length: u32, id: u8) -> Result<Message, &'static str> {
        let content = match id {
            2 => ContentType::Interested(InterestedMessage::from_bytes()),
            5 => ContentType::Bitfield(BitfieldMessage::from_bytes(&body)),
            7 | 6 => ContentType::Request(RequestMessage::from_bytes(&body)),
            20 => ContentType::Extension(ExtensionMessage::from_bytes(&body)?),
            19 => ContentType::Handshake(HandshakeMessage::from_bytes(&body)),
            _ => ContentType::Nothing(),
        };

        Ok(Message {
            id,
            length,
            content,
        })
    }

    pub fn new(content: ContentType, length: u32, id: u8) -> Message {
        Message {
            id,
            length,
            content,
        }
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_content(&self) -> &ContentType {
        &self.content
    }

    pub fn get_content_data(&self) -> Vec<u8> {
        match &self.content {
            ContentType::Nothing() => vec![],
            ContentType::Extension(extension) => extension.get_data(),
            ContentType::Request(request) => request.get_block_data(),
            ContentType::Interested(_) => vec![],
            ContentType::Bitfield(bitfield) => bitfield.get_bitfield(),
            ContentType::Handshake(_handshake) => vec![],
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let content_as_bytes = match &self.content {
            ContentType::Nothing() => vec![],
            ContentType::Extension(extension) => extension.as_bytes(),
            ContentType::Request(request) => request.as_bytes(),
            ContentType::Interested(interested) => interested.as_bytes(),
            ContentType::Bitfield(bitfield) => bitfield.as_bytes(),
            ContentType::Handshake(handshake) => return handshake.as_bytes(),
        };

        [
            self.length.to_be_bytes().to_vec(),
            self.id.to_be_bytes().to_vec(),
            content_as_bytes,
        ]
        .concat()
    }

    pub fn is_extension_data_message(&self) -> bool {
        if let ContentType::Extension(msg) = &self.content {
            if msg.is_data() {
                return true;
            }
        }
        false
    }

    pub fn is_request_message(&self) -> bool {
        matches!(self.content, ContentType::Request(_))
    }
}

#[cfg(test)]
pub fn new_generic_message(id: u8, length: u32) -> Message {
    Message::new(ContentType::Nothing(), length, id)
}

pub fn new_interested() -> Message {
    Message::new(ContentType::Nothing(), 1, 2)
}

#[cfg(test)]
pub fn new_bitfield(bitfield: Vec<u8>) -> Message {
    Message::new(
        ContentType::Bitfield(BitfieldMessage::from_bytes(&bitfield)),
        bitfield.len() as u32,
        5,
    )
}

#[cfg(test)]
pub fn new_extension(
    length: u32,
    extensions: HashMap<String, u8>,
    metadata_size: usize,
) -> Message {
    Message::new(
        ContentType::Extension(ExtensionMessage::new(extensions, metadata_size)),
        length,
        20,
    )
}

pub fn new_request(piece_index: u32, block_offset: u32, block_length: u32) -> Message {
    let content = ContentType::Request(RequestMessage::new(
        piece_index,
        block_offset,
        block_length.to_be_bytes().to_vec(),
    ));
    Message::new(content, 13, 6)
}

pub fn new_metadata(extension_id: u8, index: usize) -> Message {
    let data = format!("d8:msg_typei0e5:piecei{index}ee")
        .as_bytes()
        .to_vec();
    let length = (data.len() + 1 + 1) as u32;
    let extension = ContentType::Extension(
        ExtensionMessage::from_bytes(&vec![vec![extension_id], data].concat()).unwrap(),
    );
    Message::new(extension, length, 20)
}

pub fn new_handshake(info_hash: &[u8], peer_id: &str) -> Message {
    let content = ContentType::Handshake(HandshakeMessage::new(info_hash, peer_id));
    Message::new(content, 68, 19)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_interested() {
        let outcome = new_interested().as_bytes();
        let expect = [(1 as u32).to_be_bytes().to_vec(), vec![0x02]].concat();
        assert_eq!(outcome, expect);
    }

    #[test]
    fn test_new_request() {
        let outcome = new_request(10, 2, 4).as_bytes();

        let expect = [
            (13 as u32).to_be_bytes().to_vec(),
            [0x06].to_vec(),
            (10 as u32).to_be_bytes().to_vec(),
            (2 as u32).to_be_bytes().to_vec(),
            (4 as u32).to_be_bytes().to_vec(),
        ]
        .concat();
        assert_eq!(outcome, expect);
    }

    #[test]
    fn test_new_metadata() {
        let index = 2;
        let outcome = new_metadata(10, index).as_bytes();
        let metadata_body = format!("d8:msg_typei0e5:piecei{index}ee")
            .as_bytes()
            .to_vec();

        let expect = [vec![0x00, 0x00, 0x00, 0x1B, 0x14, 0x0A], metadata_body].concat();

        assert_eq!(outcome, expect);
    }
}
