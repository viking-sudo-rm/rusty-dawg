use std::collections::HashMap;

pub struct TokenIndex {
    // TODO: Could optimize this to only store each string once.
    token_to_index: HashMap<String, usize>,
    index_to_token: Vec<String>,
    count: usize,
    unk: usize,
}

impl TokenIndex {

    pub fn new() -> Self {
        let token_to_index = HashMap::new();
        let index_to_token = Vec::new();
        let mut index = TokenIndex {token_to_index, index_to_token, count: 0, unk: 0};
        index.add("<unk>");
        index.add("<bos>");
        index.add("<eos>");
        index
    }

    pub fn add(&mut self, token: &str) -> usize {
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

    pub fn index(&self, token: &str) -> usize {
        match self.token_to_index.get(token) {
            Some(ptr) => *ptr,
            None => self.unk,
        }
    }

    pub fn token(&self, index: usize) -> &str {
        if index < self.count {
            return self.index_to_token[index].as_str();
        }
        return self.token(self.unk);
    }

}

#[cfg(test)]
mod tests {
    use token_index::TokenIndex;

    #[test]
    fn test_token_index() {
        let mut token_index = TokenIndex::new();
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

}