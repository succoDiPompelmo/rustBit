#[derive(Debug, Clone)]
pub struct BitfieldMessage {
    bitfield: Vec<u8>,
}

impl BitfieldMessage {
    pub fn from_bytes(bytes: &[u8]) -> BitfieldMessage {
        BitfieldMessage {
            bitfield: bytes.to_vec(),
        }
    }

    pub fn get_bitfield(&self) -> Vec<u8> {
        self.bitfield.to_vec()
    }

    pub fn get_bitfield_as_bit_vector(&self) -> Vec<bool> {
        let mut bin_vector: Vec<bool> = vec![];

        for byte in &self.bitfield {
            for offset in 0..8 {
                let mask = 128 >> offset;
                if mask & byte > 0 {
                    bin_vector.push(true)
                } else {
                    bin_vector.push(false)
                }
            }
        }
        bin_vector
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.bitfield.to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input = vec![0x00, 0x01, 0x02];

        let outcome = BitfieldMessage::from_bytes(&input);
        assert_eq!(input, outcome.get_bitfield());
    }

    #[test]
    fn test_from_bytes_empty() {
        let input = vec![];

        let outcome = BitfieldMessage::from_bytes(&input);
        assert_eq!(true, outcome.get_bitfield().is_empty());
    }

    #[test]
    fn test_as_bytes() {
        let outcome = BitfieldMessage {
            bitfield: vec![0x1C, 0xAA, 0xDD, 0x0D],
        }
        .as_bytes();

        let expected = vec![0x1C, 0xAA, 0xDD, 0x0D];
        assert_eq!(expected, outcome)
    }

    #[test]
    fn test_as_bit_vector() {
        let outcome = BitfieldMessage {
            bitfield: vec![0x1C, 0xFF, 0xAA],
        }
        .get_bitfield_as_bit_vector();

        let expected = vec![
            false, false, false, true, true, true, false, false, true, true, true, true, true,
            true, true, true, true, false, true, false, true, false, true, false,
        ];
        assert_eq!(expected, outcome)
    }
}
