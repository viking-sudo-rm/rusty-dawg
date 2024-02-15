pub mod end;
pub mod null_token_index;
pub mod pretrain_tokenizer;
pub mod token_index;

pub use self::null_token_index::NullTokenIndex;
pub use self::pretrain_tokenizer::PretrainedTokenizer;
pub use self::token_index::TokenIndex;
use std::cmp::Eq;
use std::fmt::Debug;
use std::marker::Copy;

pub trait Tokenize<E>
where
    E: Eq + serde::Serialize + Copy + Debug,
{
    fn build(&mut self, text: &str);
    fn tokenize(&mut self, text: &str) -> Vec<E>;
    fn get_count(&self) -> usize;
}
