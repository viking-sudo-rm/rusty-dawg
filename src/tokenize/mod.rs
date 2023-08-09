// pub mod null_token_index;
pub mod pretrain_tokenizer;
pub mod token_index;

// pub use self::null_token_index::NullTokenIndex;
pub use self::pretrain_tokenizer::PretrainedTokenizer;
pub use self::token_index::TokenIndex;

pub trait Tokenize {
    fn build(&mut self, text: &str);
    fn tokenize(&mut self, text: &str) -> Vec<usize>;
    fn get_count(&self) -> usize;
}
