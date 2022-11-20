#[derive(Debug)]
pub struct Handshake {
    protocol_identifier_length: u8,
    protocol_identifier: Vec<u8>,
    reserved_bytes: Vec<u8>,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl Handshake {
    pub fn new(info_hash: &[u8], peer_id: &str) -> Handshake {
        Handshake {
            protocol_identifier_length: 0x13,
            protocol_identifier: "BitTorrent protocol".as_bytes().to_vec(),
            reserved_bytes: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00],
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.as_bytes().to_vec(),
        }
    }

    pub fn from_bytes(buffer: [u8; 68]) -> Handshake {
        Handshake {
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

    pub fn as_bytes(&self) -> Vec<u8> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handshake_message() {
        let outcome = Handshake::new(&vec![0x00, 0x01], "peer").as_bytes();
        let expect = vec![
            0x13, 0x42, 0x69, 0x74, 0x54, 0x6f, 0x72, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x70, 0x72,
            0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
            0x00, 0x01, 0x70, 0x65, 0x65, 0x72,
        ];

        assert_eq!(outcome, expect)
    }
}
