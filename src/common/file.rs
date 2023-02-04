use std::fs::File;
use std::io::prelude::*;

pub fn read_file(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    contents
}
