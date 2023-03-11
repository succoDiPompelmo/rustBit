use std::collections::HashMap;
use std::str::Utf8Error;
use std::{fmt, str};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Metainfo {
    Integer(usize),
    List(Vec<Metainfo>),
    String(Vec<u8>),
    Dictionary(HashMap<String, Metainfo>),
    Nothing(),
}

const METAINFO_INTEGER: &str = "Integer";
const METAINFO_LIST: &str = "List";
const METAINFO_STRING: &str = "String";
const METAINFO_DICTIONARY: &str = "Dictionary";

impl fmt::Display for Metainfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metainfo::Integer(_) => write!(f, "{METAINFO_INTEGER}"),
            Metainfo::List(_) => write!(f, "{METAINFO_LIST}"),
            Metainfo::String(_) => write!(f, "{METAINFO_STRING}"),
            Metainfo::Dictionary(_) => write!(f, "{METAINFO_DICTIONARY}"),
            Metainfo::Nothing() => write!(f, "Nothing"),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum MetainfoError {
    #[error("Expected {0} type but found {1}")]
    BadMetainfoMatch(&'static str, String),
    #[error("Utf8 string conversion failed")]
    Utf8ConversionError(#[from] Utf8Error),
    #[error("No key {0} found in dictionary")]
    NoKeyInDictionary(String),
}

impl Metainfo {
    pub fn get_bytes_content(&self) -> Result<Vec<u8>, MetainfoError> {
        match &self {
            Metainfo::String(value) => Ok(value.to_vec()),
            _ => Err(MetainfoError::BadMetainfoMatch(
                METAINFO_STRING,
                self.to_string(),
            )),
        }
    }

    pub fn get_string_content(&self) -> Result<String, MetainfoError> {
        match &self {
            Metainfo::String(value) => Ok(str::from_utf8(value)?.to_string()),
            _ => Err(MetainfoError::BadMetainfoMatch(
                METAINFO_STRING,
                self.to_string(),
            )),
        }
    }

    pub fn get_integer_content(&self) -> Result<usize, MetainfoError> {
        match &self {
            Metainfo::Integer(value) => Ok(*value),
            _ => Err(MetainfoError::BadMetainfoMatch(
                METAINFO_INTEGER,
                self.to_string(),
            )),
        }
    }

    pub fn get_list_content(&self) -> Result<&Vec<Metainfo>, MetainfoError> {
        match &self {
            Metainfo::List(value) => Ok(value),
            _ => Err(MetainfoError::BadMetainfoMatch(
                METAINFO_LIST,
                self.to_string(),
            )),
        }
    }

    pub fn get_dict_content(&self) -> Result<&HashMap<String, Metainfo>, MetainfoError> {
        match &self {
            Metainfo::Dictionary(value) => Ok(value),
            _ => Err(MetainfoError::BadMetainfoMatch(
                METAINFO_DICTIONARY,
                self.to_string(),
            )),
        }
    }

    pub fn get_value_from_dict(&self, key: &str) -> Result<&Metainfo, MetainfoError> {
        match self.get_dict_content()?.get(key) {
            Some(value) => Ok(value),
            _ => Err(MetainfoError::NoKeyInDictionary(key.to_string())),
        }
    }

    pub fn get_string_from_dict(&self, key: &str) -> Result<String, MetainfoError> {
        match self.get_dict_content()?.get(key) {
            Some(value) => value.get_string_content(),
            _ => Err(MetainfoError::NoKeyInDictionary(key.to_string())),
        }
    }

    pub fn get_integer_from_dict(&self, key: &str) -> Result<usize, MetainfoError> {
        match self.get_dict_content()?.get(key) {
            Some(value) => value.get_integer_content(),
            _ => Err(MetainfoError::NoKeyInDictionary(key.to_string())),
        }
    }

    pub fn get_list_from_dict(&self, key: &str) -> Result<&Vec<Metainfo>, MetainfoError> {
        match self.get_dict_content()?.get(key) {
            Some(value) => value.get_list_content(),
            _ => Err(MetainfoError::NoKeyInDictionary(key.to_string())),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    // #[test]
    // fn get_bytes_content_test() {
    //     let input = Metainfo::String();
    //     let output = input.get_bytes_content();
    //     // let expected_output: Result<Vec<u8>, _> = Ok(r#"Ciao"#.as_bytes().to_vec());
    //     assert_eq!(output, Ok(vec![0x00]));
    // }

    #[test]
    fn get_string_content_test() {
        let input = Metainfo::String("Ciao".to_owned().as_bytes().to_vec());
        let output = input.get_string_content();
        let expected_output = Ok("Ciao".to_owned());
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_integer_content_test() {
        let input = Metainfo::Integer(1234);
        let output = input.get_integer_content();
        let expected_output = Ok(1234);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_list_content_test() {
        let list_content = vec![
            Metainfo::Integer(12),
            Metainfo::String("ciao".as_bytes().to_vec()),
        ];
        let input = Metainfo::List(list_content.clone());
        let output = input.get_list_content();
        let expected_output = Ok(&list_content);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_string_from_dict_test() {
        let input = Metainfo::Dictionary(HashMap::from([(
            "key".to_owned(),
            Metainfo::String("value".as_bytes().to_vec()),
        )]));
        let output = input.get_string_from_dict(&"key".to_owned());

        let expected_output = Ok("value".to_owned());
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_list_from_dict_test() {
        let input = Metainfo::Dictionary(HashMap::from([(
            "key".to_owned(),
            Metainfo::List(vec![Metainfo::Integer(123)]),
        )]));
        let output = input.get_list_from_dict(&"key".to_owned());

        let vector = &vec![Metainfo::Integer(123)];
        let expected_output = Ok(vector);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_integer_from_dict_test() {
        let input =
            Metainfo::Dictionary(HashMap::from([("key".to_owned(), Metainfo::Integer(123))]));
        let output = input.get_integer_from_dict(&"key".to_owned());

        let expected_output = Ok(123);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_value_from_dict_test() {
        let input =
            Metainfo::Dictionary(HashMap::from([("key".to_owned(), Metainfo::Integer(123))]));
        let output = input.get_value_from_dict(&"key".to_owned());

        let expected_output = Ok(&Metainfo::Integer(123));
        assert_eq!(output, expected_output);
    }
}
