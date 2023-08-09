use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokenizers::tokenizer::Tokenizer;
use unicode_segmentation::UnicodeSegmentation;

use crate::tokenize::Tokenize;

pub(crate) fn tokenize(s: &str) -> impl Iterator<Item = &str> {
    s.split_word_bounds().filter(|w| {
        for c in w.chars() {
            if !c.is_whitespace() {
                return true;
            }
        }
        false
    })
}

#[derive(Debug, Clone)]
pub struct PretrainedTokenizer {
    pub tokenizer: Tokenizer,
}

impl PretrainedTokenizer {
    pub fn new(name: &str) -> Self {
        let tokenizer = Tokenizer::from_pretrained(name, None)
            .map_err(|err| anyhow!("Failed to load pretrained tokenizer {} - {}", name, err))
            .unwrap();

        PretrainedTokenizer { tokenizer }
    }
}

impl Tokenize for PretrainedTokenizer {
    fn build(&mut self, text: &str) {
        // do nothing (pretrained tokenizer is already built)
    }

    fn tokenize(&mut self, text: &str) -> Vec<usize> {
        let tokenized_text: Vec<usize> = text
            .split_whitespace()
            .map(|x| self.tokenizer.token_to_id(x).unwrap_or_default() as usize)
            .collect();
        tokenized_text
    }

    fn get_count(&self) -> usize {
        self.tokenizer.get_vocab_size(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenize2::{PretrainedTokenizer, Tokenize};
    use std::convert::TryFrom;

    #[test]
    fn test_gpt2_tokenizer() {
        let mut token_index: PretrainedTokenizer = PretrainedTokenizer::new("gpt2");
        println!("vocab size: {}", token_index.get_count());
        println!("{:?}", token_index.tokenize("hello"));

        assert_eq!(
            token_index.tokenize("hello"),
            vec! {usize::try_from(
            token_index.tokenizer.token_to_id("hello").unwrap())
            .unwrap()}
        );

        assert_eq!(token_index.get_count(), 50257);
    }
}
