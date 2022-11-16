use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time, vec};

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

    pub fn read_stream(&mut self) -> Option<(Vec<u8>, u8, u32)> {
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
            return Some((vec![], id, 0));
        } else {
            length -= 1
        }

        let mut body = vec![0; length as usize];
        for retry in 1..40 {
            if self.peek_and_read(&mut body, length as usize) {
                return Some((body, id, length));
            }
            thread::sleep(time::Duration::from_millis(5 * retry));
        }
        None
    }

    fn peek_and_read(&mut self, body: &mut [u8], length: usize) -> bool {
        self.stream.peek(body).unwrap_or(0) == length
            && self.stream.read(body).unwrap_or(0) == length
    }

    pub fn write_stream(&mut self, buffer: &[u8]) {
        self.stream
            .write_all(buffer)
            .map_err(|_| "Error in interested request")
            .unwrap();
    }

    pub fn send_metadata_handshake_request(&mut self) {
        let content = [
            &60_u32.to_be_bytes(),
            [0x14].as_slice(),
            [0x00].as_slice(),
            "d1:md11:ut_metadatai1e6:ut_pexi2ee13:metadata_sizei28282ee".as_bytes(),
        ]
        .concat();

        self.stream
            .write_all(&content)
            .map_err(|_| "Error in metadata request")
            .unwrap();
    }
}
