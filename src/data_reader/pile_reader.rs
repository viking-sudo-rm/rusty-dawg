use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

use crate::data_reader::buf_reader::BufReader;

pub struct PileReader {
    buf_reader: BufReader,
    split_map: HashMap<String, usize>,
}

fn get_pile_map() -> HashMap<String, usize> {
    // See: https://arxiv.org/pdf/2101.00027.pdf
    let splits = vec![
        "None",
        "Pile-CC",
        "PubMed Central",
        "Books3",
        "OpenWebText2",
        "ArXiv",
        "Github",
        "FreeLaw",
        "StackExchange",
        "USPTO Backgrounds",
        "PubMed Abstracts",
        "Gutenberg (PG-19)",
        "OpenSubtitles",
        "Wikipedia (en)",
        "DM Mathematics",
        "Ubuntu IRC",
        "BookCorpus2",
        "EuroParl",
        "HackerNews",
        "YoutubeSubtitles",
        "PhilPapers",
        "NIH ExPorter",
        "Enron Emails",
    ];
    let mut split_map = HashMap::new();
    for (idx, split) in splits.iter().enumerate() {
        split_map.insert(split.to_string(), idx);
    }
    split_map
}

#[derive(Serialize, Deserialize)]
struct PileDocument {
    text: String,
    meta: PileMeta,
}

#[derive(Serialize, Deserialize)]
struct PileMeta {
    pile_set_name: String,
}

impl PileReader {
    pub fn new(file: impl AsRef<std::path::Path>) -> Result<Self> {
        let buf_reader = BufReader::open(file)?;
        let split_map = get_pile_map();
        Ok(Self {
            buf_reader,
            split_map,
        })
    }
}

impl Iterator for PileReader {
    type Item = (usize, Rc<String>);

    fn next(&mut self) -> Option<(usize, Rc<String>)> {
        let opt_line = self.buf_reader.next();
        match opt_line {
            Some(line) => {
                let blob: PileDocument = serde_json::from_str(line.unwrap().as_str()).unwrap();
                let doc_id = *self.split_map.get(&blob.meta.pile_set_name).unwrap();
                let doc_text = Rc::new(blob.text);
                Some((doc_id, doc_text))
            }
            None => None,
        }
    }
}
