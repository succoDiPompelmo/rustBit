use std::collections::HashMap;

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

impl Encode for String {
    fn encode(&self) -> Vec<u8> {
        format!("{}:{}", self.chars().count(), self)
            .as_bytes()
            .to_vec()
    }
}

impl Encode for &str {
    fn encode(&self) -> Vec<u8> {
        format!("{}:{}", self.chars().count(), self)
            .as_bytes()
            .to_vec()
    }
}

impl Encode for usize {
    fn encode(&self) -> Vec<u8> {
        format!("i{}e", self).as_bytes().to_vec()
    }
}

impl Encode for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        [
            self.len().to_string().as_bytes().to_vec(),
            ":".as_bytes().to_vec(),
            self.to_vec(),
        ]
        .concat()
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self) -> Vec<u8> {
        let mut acc = "l".as_bytes().to_vec();
        for el in self {
            acc = [acc, el.encode()].concat();
        }
        [acc, "e".as_bytes().to_vec()].concat()
    }
}

impl<T: Encode> Encode for HashMap<String, T> {
    fn encode(&self) -> Vec<u8> {
        let mut keys: Vec<&String> = self.keys().collect();
        keys.sort();
        let mut acc = "d".as_bytes().to_vec();
        for key in keys {
            let value = self.get(key).unwrap();
            acc = [acc, key.encode(), value.encode()].concat();
        }
        [acc, "e".as_bytes().to_vec()].concat()
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self) -> Vec<u8> {
        match &self {
            Some(value) => value.encode(),
            None => vec![],
        }
    }
}

pub fn encode_dict_entry(key: &str, value: &impl Encode) -> Vec<u8> {
    match key {
        "" => value.encode(),
        _ => {
            let encoded_value = value.encode();
            if encoded_value.is_empty() {
                vec![]
            } else {
                [key.encode(), value.encode()].concat()
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn encode_string() {
        let input = "Bau Bau".to_owned();
        let output = input.encode();

        let expected_output = "7:Bau Bau".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_integer() {
        let input: usize = 549432982;
        let output = input.encode();

        let expected_output = "i549432982e".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_byte_vector() {
        let input: Vec<u8> = vec![0x8e, 0x99, 0x22];
        let output = input.encode();

        let expected_output = vec![51, 58, 0x8e, 0x99, 0x22];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_generic_vector() {
        let input: Vec<String> = vec!["Miao".to_owned(), "Cra".to_owned()];
        let output = input.encode();

        let expected_output = "l4:Miao3:Crae".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_string_dict_entry() {
        let output = encode_dict_entry(&"key".to_owned(), &"value".to_owned());

        let expected_output = "3:key5:value".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_integer_dict_entry() {
        let output = encode_dict_entry(&"key".to_owned(), &123);

        let expected_output = "3:keyi123e".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_list_dict_entry() {
        let output = encode_dict_entry(&"key".to_owned(), &Vec::from(["S".to_owned()]));

        let expected_output = "3:keyl1:Se".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_option_string_dict_entry() {
        let output = encode_dict_entry(&"key".to_owned(), &Some("value".to_owned()));

        let expected_output = "3:key5:value".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_option_none_dict_entry() {
        let none: Option<usize> = None;
        let output = encode_dict_entry(&"key".to_owned(), &none);

        let expected_output: Vec<u8> = vec![];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_hashmap() {
        let input = HashMap::from([
            ("Chiave".to_owned(), "Valore".to_owned()),
            ("Paolo".to_owned(), "Akunamatata".to_owned()),
        ]);
        let output = input.encode();

        let expected_output = "d6:Chiave6:Valore5:Paolo11:Akunamatatae".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_option_string() {
        let input = Some("Ciao".to_owned());
        let output = input.encode();

        let expected_output = "4:Ciao".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_option_integer() {
        let input = Some(12);
        let output = input.encode();

        let expected_output = "i12e".as_bytes();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn encode_option_none() {
        let input: Option<String> = None;
        let output = input.encode();

        let expected_output: Vec<u8> = vec![];
        assert_eq!(output, expected_output);
    }
}
