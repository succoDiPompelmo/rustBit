use std::cmp;
use std::fs;
use std::os::windows::prelude::FileExt;

use crate::torrent::file::File;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum WriterError {}

pub fn write(
    piece: Vec<u8>,
    idx: usize,
    files: Vec<File>,
    piece_length: usize,
) -> Result<(), WriterError> {
    get_file_writers(files, piece, idx, piece_length)
        .iter()
        .for_each(|writer| writer.write_to_filesystem());

    log::info!("Completed write to filesystem for piece {:?}", idx);
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileWriter {
    path: Vec<String>,
    start: usize,
    end: usize,
    piece: Vec<u8>,
}

impl FileWriter {
    fn new(path: Vec<String>, start: usize, end: usize, piece: Vec<u8>) -> FileWriter {
        FileWriter {
            path,
            start,
            end,
            piece,
        }
    }

    // TODO: Add optional folder where to save output
    pub fn write_to_filesystem(&self) {
        let path_string = &self.path.join("/");
        let path = std::path::Path::new(path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        file.seek_write(&self.piece, self.start as u64).unwrap();
    }
}

struct FileParser {
    piece_length: usize,
    offset: usize,
    start_file_index: usize,
    end_file_index: usize,
    right_file_boundary: usize,
}

impl FileParser {
    fn new(piece_length: usize, piece_index: usize, default_piece_length: usize) -> FileParser {
        let offset = piece_index * default_piece_length;
        FileParser {
            piece_length,
            offset,
            start_file_index: offset,
            right_file_boundary: 0,
            end_file_index: 0,
        }
    }

    fn update_start_file_index(&mut self) {
        self.start_file_index = self.end_file_index;
    }

    fn update_right_file_boundary(&mut self, file_length: usize) {
        self.right_file_boundary += file_length
    }

    fn update_end_file_index(&mut self) {
        self.end_file_index = cmp::min(self.right_file_boundary, self.offset + self.piece_length)
    }

    fn get_start_piece_index(&self) -> usize {
        self.start_file_index - self.offset
    }

    fn get_end_piece_index(&self) -> usize {
        self.end_file_index - self.offset
    }

    fn get_start_file_index(&self, file_length: usize) -> usize {
        self.start_file_index - (self.right_file_boundary - file_length)
    }

    fn get_end_file_index(&self, file_length: usize) -> usize {
        self.end_file_index - (self.right_file_boundary - file_length)
    }

    fn is_piece_in_file(&self) -> bool {
        self.offset < self.right_file_boundary
    }

    fn is_piece_finished(&self) -> bool {
        self.offset + self.piece_length > self.right_file_boundary
    }
}

pub fn get_file_writers(
    files: Vec<File>,
    piece: Vec<u8>,
    idx: usize,
    piece_length: usize,
) -> Vec<FileWriter> {
    let mut file_parser = FileParser::new(piece.len(), idx, piece_length);
    let mut files_to_write = vec![];

    for file in files {
        let file_length = file.get_length();
        file_parser.update_right_file_boundary(file_length);

        if file_parser.is_piece_in_file() {
            file_parser.update_end_file_index();

            files_to_write.push(FileWriter::new(
                file.get_path(),
                file_parser.get_start_file_index(file_length),
                file_parser.get_end_file_index(file_length),
                piece[file_parser.get_start_piece_index()..file_parser.get_end_piece_index()]
                    .to_vec(),
            ));

            if file_parser.is_piece_finished() {
                file_parser.update_start_file_index()
            } else {
                return files_to_write;
            }
        }
    }
    files_to_write
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proviamoci() {
        let _ = std::fs::remove_file("prova.txt");

        let piece_0: Vec<u8> = vec![0, 0, 0, 0, 0, 0];
        let piece_1: Vec<u8> = vec![1, 1, 1, 1, 1, 1];
        let writer_0 = FileWriter::new(vec!["prova.txt".to_owned()], 0, 6, piece_0);
        let writer_1 = FileWriter::new(vec!["prova.txt".to_owned()], 6, 12, piece_1);

        writer_1.write_to_filesystem();
        writer_0.write_to_filesystem();

        let output = std::fs::read("prova.txt").unwrap();
        assert_eq!(output, vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1]);

        let _ = std::fs::remove_file("prova.txt");
    }

    #[test]
    fn test_single_file_single_writer() {
        let files = vec![File::new(vec!["path".to_owned()], 32)];
        let result = get_file_writers(files, vec![0x00; 8], 2, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            16,
            24,
            vec![0x00; 8],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_single_writer() {
        let files = vec![
            File::new(vec!["path".to_owned()], 32),
            File::new(vec!["path".to_owned()], 32),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 1, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            8,
            16,
            vec![0x00; 8],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_multiple_writer() {
        let files = vec![
            File::new(vec!["path".to_owned()], 33),
            File::new(vec!["to".to_owned()], 2),
            File::new(vec!["heaven".to_owned()], 30),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 4, 8);
        let expect = vec![
            FileWriter::new(vec!["path".to_owned()], 32, 33, vec![0x00; 1]),
            FileWriter::new(vec!["to".to_owned()], 0, 2, vec![0x00; 2]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 5, vec![0x00; 5]),
        ];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_multiple_writer_edge_case() {
        let files = vec![
            File::new(vec!["path".to_owned()], 32),
            File::new(vec!["to".to_owned()], 2),
            File::new(vec!["heaven".to_owned()], 30),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 4, 8);
        let expect = vec![
            FileWriter::new(vec!["to".to_owned()], 0, 2, vec![0x00; 2]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 6, vec![0x00; 6]),
        ];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_piece_smaller_than_piece_length() {
        let files = vec![File::new(vec!["path".to_owned()], 12)];
        // Piece length should be 8
        let result = get_file_writers(files, vec![0x00; 4], 1, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            8,
            12,
            vec![0x00; 4],
        )];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_piece_too_small_ok() {
        let files = vec![File::new(vec!["path".to_owned()], 32)];
        let result = get_file_writers(files, vec![0x00; 2], 0, 2);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            0,
            2,
            vec![0x00; 2],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_big_file() {
        let files = vec![
            File::new(vec!["path".to_owned()], 1800),
            File::new(vec!["to".to_owned()], 1500),
            File::new(vec!["heaven".to_owned()], 6000),
        ];
        let result = get_file_writers(files, vec![0x00; 1700], 1, 1700);
        let expect = vec![
            FileWriter::new(vec!["path".to_owned()], 1700, 1800, vec![0x00; 100]),
            FileWriter::new(vec!["to".to_owned()], 0, 1500, vec![0x00; 1500]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 100, vec![0x00; 100]),
        ];

        assert_eq!(result, expect)
    }
}
