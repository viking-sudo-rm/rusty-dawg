use std::cmp::max;

pub struct NullTokenIndex {
    pub count: usize,
}

impl NullTokenIndex {

    pub fn new() -> Self {
        NullTokenIndex {count: 0}
    }

    pub fn add(&mut self, token: &str) -> usize {
        let index = self.index(token);
        self.count = max(self.count, index + 1);
        index
    }

    pub fn index(&self, token: &str) -> usize {
        token.parse().unwrap()
    }

}