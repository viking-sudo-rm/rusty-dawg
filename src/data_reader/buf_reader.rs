// Pete Walsh's code for a buffered read over compressed files.

use std::{
    fs::File,
    io::{self, prelude::*},
    rc::Rc,
};

use anyhow::Result;
use flate2::read::MultiGzDecoder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct DataInstance {
    // Unfortunately we can't just use a borrowed string here.
    // See https://github.com/serde-rs/serde/issues/1413#issuecomment-494892266
    pub(crate) text: Option<String>,
}

pub(crate) struct BufReader {
    reader: io::BufReader<MultiGzDecoder<File>>,
    buf: Rc<String>,
}

fn new_buf() -> Rc<String> {
    Rc::new(String::with_capacity(2048))
}

impl BufReader {
    pub(crate) fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let reader = io::BufReader::new(MultiGzDecoder::new(File::open(path)?));
        let buf = new_buf();

        Ok(Self { reader, buf })
    }
}

type DataIteratorItem = io::Result<Rc<String>>;

impl Iterator for BufReader {
    type Item = DataIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = match Rc::get_mut(&mut self.buf) {
            Some(buf) => {
                buf.clear();
                buf
            }
            None => {
                self.buf = new_buf();
                Rc::make_mut(&mut self.buf)
            }
        };

        self.reader
            .read_line(buf)
            .map(|u| {
                if u == 0 {
                    None
                } else {
                    Some(Rc::clone(&self.buf))
                }
            })
            .transpose()
    }
}