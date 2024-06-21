use anyhow::Result;
use serde_json::Value;
use std::rc::Rc;

use crate::data_reader::buf_reader::BufReader;

/// Untyped JSONL reader when text/domain are stored as unembedded keys.
pub struct JsonlReader {
    buf_reader: BufReader,
    text_key: String,
    domain_key: Option<String>,
}

impl JsonlReader {
    pub fn new(
        file: impl AsRef<std::path::Path>,
        text_key: String,
        domain_key: Option<String>,
    ) -> Result<Self> {
        let buf_reader = BufReader::open(file)?;
        Ok(Self {
            buf_reader,
            text_key,
            domain_key,
        })
    }
}

impl Iterator for JsonlReader {
    type Item = (usize, Rc<String>);

    fn next(&mut self) -> Option<(usize, Rc<String>)> {
        let opt_line = self.buf_reader.next();
        match opt_line {
            Some(line) => {
                let blob: Value = serde_json::from_str(line.unwrap().as_str()).unwrap();
                let text = blob[self.text_key.as_str()].as_str().unwrap();
                let text_rc = Rc::new(text.to_string());
                let doc_id = match self.domain_key.as_ref() {
                    // FIXME: the key is actually a string. remove this or make a hashmap
                    Some(dkey) => blob[dkey].as_u64().unwrap(),
                    None => 0,
                };
                Some((doc_id as usize, text_rc))
            }
            None => None,
        }
    }
}
