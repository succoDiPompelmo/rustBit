#[derive(Debug, Clone)]
pub struct RequestMessage {
    piece_index: u32,
    block_index: u32,
    block_data: Vec<u8>,
}

impl RequestMessage {
    pub fn new(piece_index: u32, block_index: u32, block_data: Vec<u8>) -> RequestMessage {
        RequestMessage {
            piece_index,
            block_index,
            block_data,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> RequestMessage {
        let piece_index =
            u32::from_be_bytes(bytes[..4].try_into().expect("slice with incorrect length"));
        let block_index =
            u32::from_be_bytes(bytes[4..8].try_into().expect("slice with incorrect length"));

        RequestMessage {
            piece_index,
            block_index,
            block_data: bytes[8..].to_vec(),
        }
    }

    pub fn get_block_data(&self) -> Vec<u8> {
        self.block_data.to_vec()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        [
            self.piece_index.to_be_bytes().to_vec(),
            self.block_index.to_be_bytes().to_vec(),
            self.block_data.to_vec(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_bytes_send_request() {
        let input = [
            (1 as u32).to_be_bytes(),
            (2 as u32).to_be_bytes(),
            (3 as u32).to_be_bytes(),
        ]
        .concat();

        let outcome = RequestMessage::from_bytes(&input);
        assert_eq!((3 as u32).to_be_bytes().to_vec(), outcome.get_block_data());
        assert_eq!(2, outcome.block_index);
        assert_eq!(1, outcome.piece_index);
    }

    #[test]
    fn test_from_bytes_recieve_request() {
        let data = vec![0x00, 0x32, 0x01, 0x0C];
        let input = [
            (9 as u32).to_be_bytes().to_vec(),
            (10 as u32).to_be_bytes().to_vec(),
            data.to_vec(),
        ]
        .concat();

        let outcome = RequestMessage::from_bytes(&input);
        assert_eq!(data, outcome.block_data);
        assert_eq!(10, outcome.block_index);
        assert_eq!(9, outcome.piece_index);
    }

    #[test]
    fn test_as_bytes() {
        let request = RequestMessage {
            piece_index: 1,
            block_index: 2,
            block_data: vec![0x00, 0x00, 0x00, 0x0D],
        };

        let outcome = request.as_bytes();
        let expected = vec![
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0D,
        ];
        assert_eq!(expected, outcome)
    }
}
