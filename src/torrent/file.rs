use serde::{Deserialize, Serialize};

use crate::bencode::encode::{encode_dict_entry, Encode};
use crate::bencode::metainfo::{Metainfo, MetainfoError};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct File {
    path: Vec<String>,
    length: usize,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum FileError {
    #[error("Error handling metainfo")]
    MetainfoError(#[from] MetainfoError),
}

impl File {
    pub fn from_metainfo(file: &Metainfo) -> Result<File, FileError> {
        let file_length = file.get_integer_from_dict("length")?;
        let file_path_metainfo = file.get_list_from_dict("path")?;
        let mut file_path = Vec::new();

        for a in file_path_metainfo {
            let path_value = a.get_string_content()?;
            file_path.push(path_value);
        }

        Ok(File::new(file_path, file_length))
    }

    pub fn new(path: Vec<String>, length: usize) -> File {
        File { path, length }
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_path(&self) -> Vec<String> {
        self.path.to_vec()
    }
}

impl Encode for File {
    fn encode(&self) -> Vec<u8> {
        let length = encode_dict_entry("length", &self.length);
        let path = encode_dict_entry("path", &self.path);

        [
            "d".as_bytes(),
            length.as_slice(),
            path.as_slice(),
            "e".as_bytes(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_new_file() {
        let path = vec!["path".to_owned()];
        let length = 12;

        let file = File {
            path: path.to_vec(),
            length,
        };

        assert_eq!(file, File::new(path, length));
    }

    #[test]
    fn encode_file() {
        let file = File::new(
            vec!["/bin".to_owned(), "/var".to_owned(), "/dump.txt".to_owned()],
            234,
        );

        let expected_output = b"d6:lengthi234e4:pathl4:/bin4:/var9:/dump.txtee";

        assert_eq!(file.encode(), expected_output);
    }

    #[test]
    fn encode_file_with_empty_path() {
        let path = vec![];
        let length = 12;

        let file = File {
            path: path.to_vec(),
            length,
        };

        let expected_output = b"d6:lengthi12e4:pathlee";

        assert_eq!(file.encode(), expected_output);
    }
}
