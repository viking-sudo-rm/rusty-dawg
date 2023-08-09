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
    pub fn token(&self, index: usize) -> &str {
        if index < self.count {
            return self.index_to_token[index].as_str();
        }
        return self.token(self.unk);
    }

    pub fn eos(&self) -> usize {
        2
    }
}

impl Tokenize for TokenIndex<usize> {

    fn new() -> Self {
        let token_to_index = HashMap::new();
        let index_to_token = Vec::new();
        let mut index = TokenIndex {token_to_index, index_to_token, count: 0, unk: 0};
        index.add("<unk>");
        index.add("<bos>");
        index.add("<eos>");
        index
    }

    fn tokenize(&mut self, text: &str) {
        let tokenized_text: Vec<usize> = text.split_whitespace().map(|x| self.add(x)).collect();
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
            },
        }
    }

    fn index(&self, token: &str) -> usize {
        match self.token_to_index.get(token) {
            Some(ptr) => *ptr,
            None => self.unk,
        }
    }

    fn get_count(&self) -> usize {
        self.count
    }

}

#[cfg(test)]
mod tests {
    use crate::tokenize::{Tokenize, TokenIndex};

    #[test]
    fn test_token_index() {
        let mut token_index: TokenIndex<usize> = TokenIndex::new();
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
        let mut token_index: TokenIndex<usize> = TokenIndex::new();
        token_index.tokenize("hello world");
        assert_eq!(token_index.get_count(), 5);

        token_index.tokenize("how are you ?");
        assert_eq!(token_index.get_count(), 9);
    }

}