use std::collections::HashMap;

use log::{error, info};

use crate::bencode::decode::Decoder;
use crate::bencode::encode::Encode;
use crate::bencode::metainfo::Metainfo;

#[derive(Debug, Clone)]
pub struct ExtensionMessage {
    id: u8,
    msg_type: Option<usize>,
    metadata_size: Option<usize>,
    extensions: HashMap<String, u8>,
    piece: Option<usize>,
    data: Vec<u8>,
}

impl ExtensionMessage {
    #[cfg(test)]
    pub fn new(extensions: HashMap<String, u8>, metadata_size: usize) -> ExtensionMessage {
        ExtensionMessage {
            id: 20,
            msg_type: None,
            metadata_size: Some(metadata_size),
            extensions,
            piece: None,
            data: vec![],
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ExtensionMessage, &'static str> {
        let mut decoder = Decoder::init(bytes[1..].to_vec());
        let content = match decoder.decode() {
            Ok(value) => value,
            Err(err) => {
                return {
                    error!("{:?}", err.to_string());
                    Err("Error during decoding")
                }
            }
        };

        let default = HashMap::new();
        let extensions = content
            .get_value_from_dict("m")
            .ok()
            .map(|e| e.get_dict_content())
            .map(|e| e.unwrap_or(&default))
            .map_or(HashMap::new(), build_extensions_map);

        Ok(ExtensionMessage {
            id: bytes[0],
            data: bytes[decoder.get_total_parsed_bytes() + 1..].to_vec(),
            msg_type: content.get_integer_from_dict("msg_type").ok(),
            metadata_size: content.get_integer_from_dict("metadata_size").ok(),
            piece: content.get_integer_from_dict("piece").ok(),
            extensions,
        })
    }

    pub fn get_metadata_size(&self) -> Option<usize> {
        self.metadata_size
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    pub fn get_extensions(&self) -> &HashMap<String, u8> {
        &self.extensions
    }

    pub fn is_handshake(&self) -> bool {
        self.metadata_size.is_some()
    }

    pub fn is_data(&self) -> bool {
        matches!(self.msg_type, Some(1))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut body_raw: HashMap<String, usize> = HashMap::from([]);

        if let Some(piece) = self.piece {
            body_raw.insert("piece".to_owned(), piece);
        }

        if let Some(msg_type) = self.msg_type {
            body_raw.insert("msg_type".to_owned(), msg_type);
        }

        [vec![self.id], body_raw.encode()].concat()
    }
}

fn build_extensions_map(metainfo_map: &HashMap<String, Metainfo>) -> HashMap<String, u8> {
    let mut extension: HashMap<String, u8> = HashMap::new();
    for (key, metainfo_value) in metainfo_map {
        if let Ok(value) = metainfo_value.get_integer_content() {
            extension.insert(key.to_owned(), value as u8);
        } else {
            info!("No extension value found from {:?}", metainfo_value);
        }
    }
    extension
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let outcome = ExtensionMessage {
            id: 1,
            msg_type: Some(1),
            metadata_size: None,
            extensions: HashMap::new(),
            piece: Some(2),
            data: vec![],
        }
        .as_bytes();

        let expected = [&vec![0x01], "d8:msg_typei1e5:piecei2ee".as_bytes()].concat();
        assert_eq!(expected, outcome)
    }

    #[test]
    fn test_from_bytes_handshake() {
        let id: u8 = 20;
        let input = [vec![id], b"d13:metadata_sizei1024e1:md3:fooi2eee".to_vec()].concat();

        let outcome = ExtensionMessage::from_bytes(&input).unwrap();
        assert_eq!(Some(1024), outcome.get_metadata_size());
        assert_eq!(
            &HashMap::from([("foo".to_owned(), 2)]),
            outcome.get_extensions()
        );
        assert_eq!(true, outcome.get_data().is_empty());
        assert_eq!(None, outcome.msg_type);
    }

    #[test]
    fn test_from_bytes_data() {
        let id: u8 = 20;
        let data: Vec<u8> = vec![0x11, 0x22];
        let input = [
            vec![id],
            b"d8:msg_typei1e5:piecei2ee".to_vec(),
            data.to_vec(),
        ]
        .concat();

        let outcome = ExtensionMessage::from_bytes(&input).unwrap();
        assert_eq!(None, outcome.get_metadata_size());
        assert_eq!(&HashMap::from([]), outcome.get_extensions());
        assert_eq!(data, outcome.get_data());
        assert_eq!(Some(1), outcome.msg_type);
        assert_eq!(Some(2), outcome.piece);
    }
}
