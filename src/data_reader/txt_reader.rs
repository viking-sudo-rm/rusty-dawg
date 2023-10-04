use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read};
use std::rc::Rc;

pub struct TxtReader {
    buf_reader: BufReader<File>,
    buffer: Vec<u8>,
    split_token: Option<String>,
    docs: VecDeque<Rc<String>>,
    counter: usize,
}

impl TxtReader {
    pub fn new(file: File, buf_size: usize, split_token: Option<String>) -> Self {
        let buf_reader = BufReader::with_capacity(buf_size, file);
        let buffer = vec![0; buf_size];
        let docs: VecDeque<Rc<String>> = VecDeque::new();
        Self {
            buf_reader,
            buffer,
            split_token,
            docs,
            counter: 0,
        }
    }

    // Returned value represents whether anything was read.
    pub fn refill_buffer(&mut self) -> Result<bool> {
        let n_bytes_read = self.buf_reader.read(&mut self.buffer).unwrap();
        if n_bytes_read == 0 {
            return Ok(false);
        }

        let text = std::str::from_utf8(&self.buffer)?;
        match self.split_token.clone() {
            Some(token) => {
                for doc in text.split(&token) {
                    self.docs.push_back(Rc::new(doc.to_string()));
                }
            }
            None => {
                self.docs.push_back(Rc::new(text.to_string()));
            }
        }
        Ok(true)
    }
}

impl Iterator for TxtReader {
    type Item = (usize, Rc<String>);

    fn next(&mut self) -> Option<(usize, Rc<String>)> {
        if !self.docs.is_empty() || self.refill_buffer().unwrap() {
            let doc = self.docs.pop_front().unwrap();
            let counter = self.counter;
            self.counter += 1;
            Some((counter, doc.clone()))
        } else {
            None
        }
    }
}
