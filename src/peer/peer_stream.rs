use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
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
        if let Err(_) = self.stream.read(&mut buffer) {
            return None;
        }
        let mut length = u32::from_be_bytes(buffer);

        let mut buffer: [u8; 1] = [0x00; 1];
        if let Err(_) = self.stream.read(&mut buffer) {
            return None;
        }
        let id = buffer[0];

        if length == 0 {
            Some(Message::new(ContentType::Nothing(), length, id));
        } else {
            length = length - 1
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

    fn peek_and_read(&mut self, body: &mut Vec<u8>, length: usize) -> bool {
        self.stream.peek(body).unwrap_or(0) == length
            && self.stream.read(body).unwrap_or(0) == length
    }

    pub fn send_interested(&mut self) {
        let bytes_written = self
            .stream
            .write(&new_interested().as_bytes())
            .or_else(|_| return Err("Error in interested request"));
    }

    pub fn send_request(&mut self, block_length: u32, block_offset: u32, piece_index: u32) {
        let bytes_written = self
            .stream
            .write(&new_request(piece_index, block_offset, block_length).as_bytes())
            .or_else(|_| return Err("Error in piece request"));
    }

    pub fn send_metadata_request(&mut self, extension_id: u8, index: usize) {
        let bytes_written = self
            .stream
            .write(&new_metadata(extension_id, index).as_bytes())
            .or_else(|_| return Err("Error in metadata request"));
    }
}
