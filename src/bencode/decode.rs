use std::collections::HashMap;
use std::str;

use log::{error};

use crate::bencode::metainfo::Metainfo;

pub struct Decoder {
    current: usize,
    contents: Vec<u8>,
}

impl Decoder {
    pub fn init(source: Vec<u8>) -> Decoder {
        Decoder {
            current: 0,
            contents: source,
        }
    }

    fn advance(&mut self) -> usize {
        self.current += 1;
        self.current
    }

    fn get_current_byte(&self) -> u8 {
        self.contents[self.current]
    }

    fn get_consecutive_digits(&mut self) -> Result<usize, &'static str> {
        let start = self.current;
        while self.get_current_byte().is_ascii_digit() {
            self.advance();
        }
        let end = self.current;

        let string_integer = match str::from_utf8(&self.contents[start..end]) {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Error during string conversion at position {:?}",
                    self.current
                );
                return Err("Error during string conversion at position");
            }
        };

        match string_integer.parse() {
            Ok(result) => Ok(result),
            Err(_) => {
                error!(
                    "Error during integer conversion at position {:?}",
                    self.current
                );
                Err("Error during integer conversion")
            }
        }
    }

    fn parse_integer(&mut self) -> Result<Metainfo, &'static str> {
        self.advance();
        let integer: usize = self.get_consecutive_digits()?;

        match self.contents.get(self.current) {
            Some(b'e') => self.advance(),
            _ => self.advance(),
        };
        Ok(Metainfo::Integer(integer))
    }

    fn parse_string(&mut self) -> Result<Metainfo, &'static str> {
        let integer = self.get_consecutive_digits()?;
        // TODO: Mettere check semicolon
        self.advance();

        let b = &self.contents[self.current..self.current + integer];
        self.current += integer;
        Ok(Metainfo::String(b.to_vec()))
    }

    fn parse_list(&mut self) -> Result<Metainfo, &'static str> {
        self.advance();
        let mut list: Vec<Metainfo> = Vec::new();

        while self.get_current_byte() != b'e' {
            list.push(self.decode()?);
        }

        self.advance();
        Ok(Metainfo::List(list))
    }

    fn parse_dictionary(&mut self) -> Result<Metainfo, &'static str> {
        self.advance();

        let mut dictionary = HashMap::new();
        while self.get_current_byte() != b'e' {
            let key = if let Metainfo::String(raw_key) = self.parse_string()? {
                str::from_utf8(&raw_key)
                    .map_err(|_| "Error in string conversion")?
                    .to_owned()
            } else {
                return Err("Error in string parsing");
            };
            let value = self.decode()?;
            dictionary.insert(key, value);
        }

        self.advance();
        Ok(Metainfo::Dictionary(dictionary))
    }

    pub fn decode(&mut self) -> Result<Metainfo, &'static str> {
        let result = match self.contents.get(self.current) {
            None => Metainfo::Nothing(),
            Some(b'i') => self.parse_integer()?,
            Some(b'l') => self.parse_list()?,
            Some(b'd') => self.parse_dictionary()?,
            Some(b'1'..=b'9') => self.parse_string()?,
            Some(_) => Metainfo::Nothing(),
        };

        Ok(result)
    }

    pub fn get_total_parsed_bytes(&self) -> usize {
        self.current
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_simple_string() {
        let mut decoder = Decoder::init("3:bau".as_bytes().to_vec());
        let output = decoder.decode().unwrap();

        assert_eq!(output, Metainfo::String("bau".as_bytes().to_vec()));
    }

    #[test]
    fn decode_simple_integer() {
        let mut decoder = Decoder::init("i38e".as_bytes().to_vec());
        let output = decoder.decode().unwrap();

        assert_eq!(output, Metainfo::Integer(38));
    }

    #[test]
    fn decode_simple_list() {
        let mut decoder = Decoder::init("l4:miaoi38ee".as_bytes().to_vec());
        let output = decoder.decode().unwrap();

        assert_eq!(
            output,
            Metainfo::List(vec![
                Metainfo::String("miao".as_bytes().to_vec()),
                Metainfo::Integer(38)
            ])
        );
    }

    #[test]
    fn decode_simple_dictionary() {
        let mut decoder = Decoder::init("d4:miaoi38ee".as_bytes().to_vec());
        let output = decoder.decode().unwrap();

        assert_eq!(
            output,
            Metainfo::Dictionary(HashMap::from([("miao".to_owned(), Metainfo::Integer(38))]))
        );
    }

    #[test]
    fn decode_complex_content() {
        let mut decoder = Decoder::init("d4:miaoi38e4:infod5:peersi18eee".as_bytes().to_vec());
        let output = decoder.decode().unwrap();

        assert_eq!(
            output,
            Metainfo::Dictionary(HashMap::from([
                ("miao".to_owned(), Metainfo::Integer(38)),
                (
                    "info".to_owned(),
                    Metainfo::Dictionary(HashMap::from([(
                        "peers".to_owned(),
                        Metainfo::Integer(18)
                    )]))
                )
            ]))
        );
    }

    #[test]
    fn decode_double_content() {
        let mut decoder = Decoder::init("d3:fooi32eed3:bar3:booe".as_bytes().to_vec());
        let first_output = decoder.decode().unwrap();

        assert_eq!(
            first_output,
            Metainfo::Dictionary(HashMap::from([("foo".to_owned(), Metainfo::Integer(32))]))
        );

        let second_output = decoder.decode().unwrap();

        assert_eq!(
            second_output,
            Metainfo::Dictionary(HashMap::from([(
                "bar".to_owned(),
                Metainfo::String("boo".to_owned().as_bytes().to_vec())
            )]))
        );
    }
}
