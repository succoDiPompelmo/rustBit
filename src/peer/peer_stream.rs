use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time};

use crate::messages::{new_interested, new_metadata, new_request, ContentType, Message};

#[derive(Debug)]
pub struct PeerStream {
    stream: TcpStream,
}

impl PeerStream {
    pub fn new(stream: TcpStream) -> PeerStream {
        PeerStream { stream }
    }

    fn has_messages(&self) -> bool {
        let mut buffer: [u8; 1] = [0x00; 1];

        match self.stream.peek(&mut buffer) {
            Ok(0) => false,
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn read_message(&mut self) -> Option<Message> {
        if !self.has_messages() {
            return None;
        }

        let mut buffer: [u8; 4] = [0x00; 4];
        if self.stream.read(&mut buffer).is_err() {
            return None;
        }
        let mut length = u32::from_be_bytes(buffer);

        let mut buffer: [u8; 1] = [0x00; 1];
        if self.stream.read(&mut buffer).is_err() {
            return None;
        }
        let id = buffer[0];

        if length == 0 {
            return Some(Message::new(ContentType::Nothing(), length, id));
        } else {
            length -= 1
        }

        let mut body = vec![0; length as usize];
        for retry in 1..40 {
            if self.peek_and_read(&mut body, length as usize) {
                return Some(Message::new_raw(body, length, id));
            }
            thread::sleep(time::Duration::from_millis(5 * retry));
        }
        None
    }

    fn peek_and_read(&mut self, body: &mut [u8], length: usize) -> bool {
        self.stream.peek(body).unwrap_or(0) == length
            && self.stream.read(body).unwrap_or(0) == length
    }

    // Generic message and its function to transform into as_bytes

    pub fn send_interested(&mut self) {
        self.stream
            .write_all(&new_interested().as_bytes())
            .map_err(|_| "Error in interested request")
            .unwrap();
    }

    pub fn send_request(&mut self, block_length: u32, block_offset: u32, piece_index: u32) {
        self.stream
            .write_all(&new_request(piece_index, block_offset, block_length).as_bytes())
            .map_err(|_| "Error in piece request")
            .unwrap();
    }

    pub fn send_metadata_request(&mut self, extension_id: u8, index: usize) {
        self.stream
            .write_all(&new_metadata(extension_id, index).as_bytes())
            .map_err(|_| "Error in metadata request")
            .unwrap();
    }

    // pub fn send_metadata_handshake_request(&mut self) {
    //     let content = [
    //         &60_u32.to_be_bytes(),
    //         [0x14].as_slice(),
    //         [0x00].as_slice(),
    //         "d1:md11:ut_metadatai1e6:ut_pexi2ee13:metadata_sizei28282ee".as_bytes(),
    //     ]
    //     .concat();

    //     self.stream
    //         .write_all(&content)
    //         .map_err(|_| "Error in metadata request")
    //         .unwrap();
    // }
}
