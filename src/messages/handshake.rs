use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::tracker::tracker::PeerConnectionInfo;

#[derive(Debug)]
pub struct HandshakeMessage {
    protocol_identifier_length: u8,
    protocol_identifier: Vec<u8>,
    reserved_bytes: Vec<u8>,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl HandshakeMessage {
    pub fn new(info_hash: &[u8], peer_id: &str) -> HandshakeMessage {
        HandshakeMessage {
            protocol_identifier_length: 0x13,
            protocol_identifier: "BitTorrent protocol".as_bytes().to_vec(),
            reserved_bytes: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00],
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.as_bytes().to_vec(),
        }
    }

    pub fn from_bytes(buffer: [u8; 68]) -> HandshakeMessage {
        HandshakeMessage {
            protocol_identifier_length: buffer[0],
            protocol_identifier: buffer[1..20].to_vec(),
            reserved_bytes: buffer[20..28].to_vec(),
            info_hash: buffer[28..48].to_vec(),
            peer_id: buffer[48..68].to_vec(),
        }
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.to_vec()
    }

    fn as_bytes(&self) -> Vec<u8> {
        [
            vec![self.protocol_identifier_length],
            self.protocol_identifier.to_vec(),
            self.reserved_bytes.to_vec(),
            self.info_hash.to_vec(),
            self.peer_id.to_vec(),
        ]
        .concat()
    }
}

pub fn perform(
    peer_info: PeerConnectionInfo,
    info_hash: &Vec<u8>,
    peer_id: &str,
) -> Result<TcpStream, &'static str> {
    let peer_url = format!("{}:{}", peer_info.ip, peer_info.port);
    let server: SocketAddr = peer_url.parse().expect("Unable to parse socket address");

    let connect_timeout = Duration::from_secs(2);
    let stream = TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")?;

    send_handshake(&stream, info_hash, peer_id)?;
    let handshake_response_message = read_handshake(&stream)?;

    if handshake_response_message.get_info_hash() != *info_hash {
        return Err("Info hash not matching in handshake response");
    }

    // if (handshake_response_message.get_reserved_bytes()[5] & 0x10 == 0x10) {
    //     println!("EXTENSION SUPPORTED");
    // }

    Ok(stream)
}

fn read_handshake(mut stream: &TcpStream) -> Result<HandshakeMessage, &'static str> {
    let mut buffer: [u8; 68] = [0x00; 68];

    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    match stream.read(&mut buffer) {
        Ok(68) => Ok(HandshakeMessage::from_bytes(buffer)),
        _ => Err("Error reading handshake response"),
    }
}

fn send_handshake(
    mut stream: &TcpStream,
    info_hash: &Vec<u8>,
    peer_id: &str,
) -> Result<(), &'static str> {
    match stream.write(&HandshakeMessage::new(info_hash, peer_id).as_bytes()) {
        Ok(68) => Ok(()),
        _ => Err("Error sending handshake request"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handshake_message() {
        let outcome = HandshakeMessage::new(&vec![0x00, 0x01], "peer").as_bytes();
        let expect = vec![
            0x13, 0x42, 0x69, 0x74, 0x54, 0x6f, 0x72, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x70, 0x72,
            0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
            0x00, 0x01, 0x70, 0x65, 0x65, 0x72,
        ];

        assert_eq!(outcome, expect)
    }
}
