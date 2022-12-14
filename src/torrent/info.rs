use sha1::{Digest, Sha1};

use crate::bencode::decode::Decoder;
use crate::bencode::encode::{encode_dict_entry, Encode};
use crate::bencode::metainfo;
use crate::bencode::metainfo::Metainfo;
use crate::torrent::file::File;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Info {
    files: Option<Vec<File>>,
    length: Option<usize>,
    name: String,
    piece_length: usize,
    pieces: Vec<u8>,
}

impl Info {
    #[cfg(test)]
    pub fn new(
        files: Option<Vec<File>>,
        length: Option<usize>,
        name: String,
        piece_length: usize,
        pieces: Vec<u8>,
    ) -> Info {
        Info {
            files,
            length,
            name,
            piece_length,
            pieces,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Info, &'static str> {
        let decoded_info = Decoder::init(bytes).decode()?;
        Info::from_metainfo(&decoded_info)
    }

    pub fn from_metainfo(a: &Metainfo) -> Result<Info, &'static str> {
        let pieces = match metainfo::get_value_from_dict(a, "pieces")? {
            Metainfo::String(pieces) => pieces,
            _ => return Err("No pieces found"),
        };
        let piece_length = metainfo::get_integer_from_dict(a, "piece length")?;
        let name = metainfo::get_string_from_dict(a, "name")?;
        let length = metainfo::get_integer_from_dict(a, "length").ok();

        let files = match metainfo::get_list_from_dict(a, "files") {
            Ok(metainfo_files) => {
                let mut output_files = Vec::new();
                for metainfo_file in metainfo_files {
                    output_files.push(File::from_metainfo(metainfo_file)?)
                }
                Some(output_files)
            }
            Err(_) => None,
        };

        Ok(Info {
            name,
            pieces: pieces.to_vec(),
            piece_length,
            files,
            length,
        })
    }

    pub fn get_piece(&self, index: usize) -> &[u8] {
        self.pieces
            .chunks_exact(20)
            .nth(index)
            .expect("No piece at the provided index")
    }

    pub fn verify_piece(&self, piece: &[u8], piece_idx: usize) -> bool {
        Sha1::digest(piece).as_slice() == self.get_piece(piece_idx)
    }

    pub fn get_piece_length(&self) -> usize {
        self.piece_length
    }

    pub fn get_total_length(&self) -> usize {
        match &self.files {
            Some(files) => files.iter().map(|file| file.get_length()).sum::<usize>(),
            None => self.length.unwrap(),
        }
    }

    pub fn get_files(&self) -> Result<Vec<File>, &'static str> {
        match &self.files {
            Some(files) => Ok(files.to_vec()),
            None => {
                let file = File::new(vec![self.name.to_owned()], self.length.unwrap());
                Ok(vec![file])
            }
        }
    }

    pub fn compute_info_hash(&self) -> Vec<u8> {
        Sha1::digest(self.encode()).as_slice().to_owned()
    }
}

impl Encode for Info {
    fn encode(&self) -> Vec<u8> {
        let files = encode_dict_entry("files", &self.files);
        let length = encode_dict_entry("files", &self.length);
        let name = encode_dict_entry("name", &self.name);
        let piece_length = encode_dict_entry("piece length", &self.piece_length);
        let pieces = encode_dict_entry("pieces", &self.pieces);

        [
            "d".as_bytes(),
            files.as_slice(),
            length.as_slice(),
            name.as_slice(),
            piece_length.as_slice(),
            pieces.as_slice(),
            "e".as_bytes(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn encode_info() {
        let file = File::new(vec!["/bin".to_owned()], 234);

        let info = Info {
            name: "pippo".to_owned(),
            piece_length: 43921,
            pieces: "ABCDE".as_bytes().to_vec(),
            files: Some(Vec::from([file])),
            length: None,
        };

        let expected_hash = "d5:filesld6:lengthi234e4:pathl4:/bineee4:name5:pippo12:piece lengthi43921e6:pieces5:ABCDEe";
        let result_hash = info.encode();

        assert_eq!(expected_hash.as_bytes().to_vec(), result_hash);
    }
}
