use std::collections::HashMap;
use std::str;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Metainfo {
    Integer(usize),
    List(Vec<Metainfo>),
    String(Vec<u8>),
    Dictionary(HashMap<String, Metainfo>),
    Nothing(),
}

impl Metainfo {
    pub fn get_bytes_content(&self) -> Result<Vec<u8>, &'static str> {
        match &self {
            Metainfo::String(value) => Ok(value.to_vec()),
            _ => Err("No bytes metainfo found"),
        }
    }

    pub fn get_string_content(&self) -> Result<String, &'static str> {
        match &self {
            Metainfo::String(value) => str::from_utf8(value)
                .map(|el| el.to_string())
                .map_err(|_| "Error during UTF-8 string conversion"),
            _ => Err("No string metainfo found"),
        }
    }

    pub fn get_integer_content(&self) -> Result<usize, &'static str> {
        match &self {
            Metainfo::Integer(value) => Ok(*value),
            _ => Err("No numeric metainfo found"),
        }
    }

    pub fn get_list_content(&self) -> Result<&Vec<Metainfo>, &'static str> {
        match &self {
            Metainfo::List(value) => Ok(value),
            _ => Err("No list metainfo found"),
        }
    }

    pub fn get_dict_content(&self) -> Result<&HashMap<String, Metainfo>, &'static str> {
        match &self {
            Metainfo::Dictionary(value) => Ok(value),
            _ => Err("No dict metainfo found"),
        }
    }

    pub fn get_value_from_dict(&self, key: &str) -> Result<&Metainfo, &'static str> {
        match self.get_dict_content()?.get(key) {
            Some(value) => Ok(value),
            _ => Err("No key found in dict"),
        }
    }

    pub fn get_string_from_dict(&self, key: &str) -> Result<String, &'static str> {
        match self.get_dict_content()?.get(key) {
            Some(value) => value.get_string_content(),
            _ => Err("No key found in dict"),
        }
    }

    pub fn get_integer_from_dict(&self, key: &str) -> Result<usize, &'static str> {
        match self.get_dict_content()?.get(key) {
            Some(Metainfo::Integer(value)) => Ok(*value),
            _ => Err("No key found in dict"),
        }
    }

    pub fn get_list_from_dict(&self, key: &str) -> Result<&Vec<Metainfo>, &'static str> {
        match self.get_dict_content()?.get(key) {
            Some(Metainfo::List(value)) => Ok(value),
            _ => Err("No key found in dict"),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

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
