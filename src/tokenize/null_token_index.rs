use std::cmp::max;

use crate::tokenize::Tokenize;

pub struct NullTokenIndex {
    pub count: usize,
}

impl Tokenize for NullTokenIndex {
    fn new() -> Self {
        NullTokenIndex { count: 0 }
    }

    fn add(&mut self, token: &str) -> usize {
        let index = self.index(token);
        self.count = max(self.count, index + 1);
        index
    }

    fn index(&self, token: &str) -> usize {
        token.parse().unwrap()
    }

    fn get_count(&self) -> usize {
        self.count
    }
}
