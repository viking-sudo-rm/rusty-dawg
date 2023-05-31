use std::collections::HashMap;

pub struct TokenIndex {
    // TODO: Could optimize this to only store each string once.
    token_to_index: HashMap<String, usize>,
    index_to_token: Vec<String>,
    count: usize,
}

impl TokenIndex {

    pub fn new() -> Self {
        let token_to_index = HashMap::new();
        let index_to_token = Vec::new();
        TokenIndex {token_to_index, index_to_token, count: 0}
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

    pub fn token_to_index(&self, token: &str) -> Option<usize> {
        match self.token_to_index.get(token) {
            Some(ptr) => Some(*ptr),
            None => None,
        }
    }

    pub fn index_to_token(&self, index: usize) -> &str {
        self.index_to_token[index].as_str()
    }

}

#[cfg(test)]
mod tests {
    use token_index::TokenIndex;

    #[test]
    fn test_token_index() {
        let mut token_index = TokenIndex::new();
        assert_eq!(token_index.add("hello"), 0);
        assert_eq!(token_index.add("hello"), 0);
        assert_eq!(token_index.add("world"), 1);
        assert_eq!(token_index.add("hello"), 0);
        assert_eq!(token_index.token_to_index("hello").unwrap(), 0);
        assert_eq!(token_index.index_to_token(0), "hello");
        assert_eq!(token_index.token_to_index("world").unwrap(), 1);
        assert_eq!(token_index.index_to_token(1), "world");
    }

}