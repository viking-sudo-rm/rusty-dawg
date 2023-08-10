use std::collections::HashMap;

use crate::tokenize::Tokenize;

pub struct TokenIndex<E> {
    // TODO: Could optimize this to only store each string once.
    // TODO: Make token type generic.
    token_to_index: HashMap<String, E>,
    index_to_token: Vec<String>,
    pub count: usize,
    unk: E,
}

impl TokenIndex<usize> {
    pub fn new() -> Self {
        let token_to_index = HashMap::new();
        let index_to_token = Vec::new();
        let mut index = TokenIndex {
            token_to_index,
            index_to_token,
            count: 0,
            unk: 0,
        };
        index.add("<unk>");
        index.add("<bos>");
        index.add("<eos>");
        index
    }

    pub fn token(&self, index: usize) -> &str {
        if index < self.count {
            return self.index_to_token[index].as_str();
        }
        return self.token(self.unk);
    }

    pub fn eos(&self) -> usize {
        2
    }

    fn add(&mut self, token: &str) -> usize {
        let token_string = token.to_string();
        match self.token_to_index.get(token) {
            Some(ptr) => *ptr,
            None => {
                self.token_to_index.insert(token_string, self.count);
                // TODO: Could optimize this to only store each string once.
                self.index_to_token.push(token.to_string());
                self.count += 1;
                self.count - 1
            }
        }
    }

    fn index(&self, token: &str) -> u16 {
        match self.token_to_index.get(token) {
            Some(ptr) => *ptr as u16, // Convert usize to u16
            None => self.unk as u16,  // Convert usize to u16
        }
    }
    
}

impl Tokenize for TokenIndex<usize> {
    fn build(&mut self, text: &str) {
        let tokens: Vec<usize> = text.split_whitespace().map(|x| self.add(x)).collect();
    }

    fn tokenize(&mut self, text: &str) -> Vec<u16> {
        let tokenized_text: Vec<u16> = text.split_whitespace().map(|x| self.index(x)).collect();
        tokenized_text
    }

    fn get_count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenize::{TokenIndex, Tokenize};

    #[test]
    fn test_build_tokenizer() {
        let mut token_index: TokenIndex<u16> = TokenIndex::new();
        token_index.build("");
        assert_eq!(token_index.get_count(), 3);

        let mut token_index: TokenIndex<u16> = TokenIndex::new();
        token_index.build("hello");
        assert_eq!(token_index.get_count(), 4);

        let mut token_index: TokenIndex<u16> = TokenIndex::new();
        token_index.build("hello, this is me.");
        assert_eq!(token_index.get_count(), 7);
    }

    #[test]
    fn test_add() {
        let mut token_index: TokenIndex<u16> = TokenIndex::new();
        assert_eq!(token_index.add("hello"), 3);
        assert_eq!(token_index.add("hello"), 3);
        assert_eq!(token_index.add("world"), 4);
        assert_eq!(token_index.add("hello"), 3);
        assert_eq!(token_index.index("hello"), 3);
        assert_eq!(token_index.token(3), "hello");
        assert_eq!(token_index.index("world"), 4);
        assert_eq!(token_index.token(4), "world");
        assert_eq!(token_index.token(0), "<unk>");
        assert_eq!(token_index.token(342), "<unk>");
        assert_eq!(token_index.index("<unk>"), 0);
        assert_eq!(token_index.index("universe"), 0);
    }

    #[test]
    fn test_tokenize_fn() {
        let mut token_index: TokenIndex<u16> = TokenIndex::new();
        token_index.build("");
        let tokens = token_index.tokenize("hello world");
        assert_eq!(
            tokens,
            "<unk> <unk>"
                .split_whitespace()
                .map(|x| token_index.index(x))
                .collect::<Vec<u16>>()
        );

        token_index.build("hello wolrd");
        let tokens = token_index.tokenize("hello world");
        assert_eq!(
            tokens,
            "hello world"
                .split_whitespace()
                .map(|x| token_index.index(x))
                .collect::<Vec<u16>>()
        );
    }
}
