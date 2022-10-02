use std::collections::HashMap;
use std::str;

#[derive(Debug, PartialEq, Clone)]
pub enum Metainfo {
    Integer(usize),
    List(Vec<Metainfo>),
    String(Vec<u8>),
    Dictionary(HashMap<String, Metainfo>),
    Nothing(),
}

pub fn get_string_content(metainfo_string: &Metainfo) -> Result<String, &'static str> {
    if let Metainfo::String(string_value) = metainfo_string {
        match str::from_utf8(string_value) {
            Ok(string) => Ok(string.to_owned()),
            Err(_) => Err("Error during UTF-8 string conversion"),
        }
    } else {
        Err("No string metainfo found")
    }
}

pub fn get_integer_content(metainfo_string: &Metainfo) -> Result<usize, &'static str> {
    if let Metainfo::Integer(integer_value) = metainfo_string {
        Ok(*integer_value)
    } else {
        Err("No string metainfo found")
    }
}

pub fn get_list_content(metainfo_list: &Metainfo) -> Result<&Vec<Metainfo>, &'static str> {
    if let Metainfo::List(list_values) = metainfo_list {
        Ok(&list_values)
    } else {
        Err("No list metainfo found")
    }
}

pub fn get_dict_content(
    metainfo_dict: &Metainfo,
) -> Result<&HashMap<String, Metainfo>, &'static str> {
    if let Metainfo::Dictionary(dict) = metainfo_dict {
        Ok(dict)
    } else {
        Err("No dict metainfo found")
    }
}

pub fn get_value_from_dict<'a>(
    metainfo_dict: &'a Metainfo,
    key: &str,
) -> Result<&'a Metainfo, &'static str> {
    let dict = get_dict_content(metainfo_dict)?;
    if let Some(value_metainfo) = dict.get(key) {
        Ok(value_metainfo)
    } else {
        Err("No key found in dict")
    }
}

pub fn get_string_from_dict(metainfo_dict: &Metainfo, key: &str) -> Result<String, &'static str> {
    let dict = get_dict_content(metainfo_dict)?;
    if let Some(key_metainfo) = dict.get(key) {
        get_string_content(key_metainfo)
    } else {
        Err("No key found in dict")
    }
}

pub fn get_integer_from_dict(metainfo_dict: &Metainfo, key: &str) -> Result<usize, &'static str> {
    let dict = get_dict_content(metainfo_dict)?;
    if let Some(Metainfo::Integer(key_value)) = dict.get(key) {
        Ok(*key_value)
    } else {
        Err("No key foundin dict")
    }
}

pub fn get_list_from_dict<'a>(
    metainfo_dict: &'a Metainfo,
    key: &str,
) -> Result<&'a Vec<Metainfo>, &'static str> {
    let dict = get_dict_content(metainfo_dict)?;

    if let Some(Metainfo::List(key_values)) = dict.get(key) {
        Ok(&key_values)
    } else {
        Err("No key foundin dict")
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn get_string_content_test() {
        let input = Metainfo::String("Ciao".to_owned().as_bytes().to_vec());
        let output = get_string_content(&input);
        let expected_output = Ok("Ciao".to_owned());
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_integer_content_test() {
        let input = Metainfo::Integer(1234);
        let output = get_integer_content(&input);
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
        let output = get_list_content(&input);
        let expected_output = Ok(&list_content);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_string_from_dict_test() {
        let input = Metainfo::Dictionary(HashMap::from([(
            "key".to_owned(),
            Metainfo::String("value".as_bytes().to_vec()),
        )]));
        let output = get_string_from_dict(&input, &"key".to_owned());

        let expected_output = Ok("value".to_owned());
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_list_from_dict_test() {
        let input = Metainfo::Dictionary(HashMap::from([(
            "key".to_owned(),
            Metainfo::List(vec![Metainfo::Integer(123)]),
        )]));
        let output = get_list_from_dict(&input, &"key".to_owned());

        let vector = &vec![Metainfo::Integer(123)];
        let expected_output = Ok(vector);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_integer_from_dict_test() {
        let input =
            Metainfo::Dictionary(HashMap::from([("key".to_owned(), Metainfo::Integer(123))]));
        let output = get_integer_from_dict(&input, &"key".to_owned());

        let expected_output = Ok(123);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn get_value_from_dict_test() {
        let input =
            Metainfo::Dictionary(HashMap::from([("key".to_owned(), Metainfo::Integer(123))]));
        let output = get_value_from_dict(&input, &"key".to_owned());

        let expected_output = Ok(&Metainfo::Integer(123));
        assert_eq!(output, expected_output);
    }
}
