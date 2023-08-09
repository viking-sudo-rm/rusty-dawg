pub mod null_token_index;
pub mod token_index;

pub use self::null_token_index::NullTokenIndex;
pub use self::token_index::TokenIndex;

pub trait Tokenize {
    fn new() -> Self
    where
        Self: Sized;
    fn add(&mut self, token: &str) -> usize;
    fn index(&self, token: &str) -> usize;
    fn get_count(&self) -> usize;
}
