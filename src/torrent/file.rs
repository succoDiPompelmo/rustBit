use crate::bencode::encode::{encode_dict_entry, Encode};
use crate::bencode::metainfo;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct File {
    path: Vec<String>,
    length: usize,
}

impl File {
    pub fn from_metainfo(file: &metainfo::Metainfo) -> Result<File, &'static str> {
        let file_length = metainfo::get_integer_from_dict(file, "length")?;
        let file_path_metainfo = metainfo::get_list_from_dict(file, "path")?;
        let mut file_path = Vec::new();

        for a in file_path_metainfo {
            let path_value = metainfo::get_string_content(a)?;
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
        let length = encode_dict_entry(&"length".to_owned(), &self.length);
        let path = encode_dict_entry(&"path".to_owned(), &self.path);

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
