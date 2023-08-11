use std::cmp::max;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;

use crate::tokenize::Tokenize;

pub struct NullTokenIndex {
    pub count: usize,
}

impl Default for NullTokenIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl NullTokenIndex {
    pub fn new() -> Self {
        NullTokenIndex { count: 0 }
    }

    fn index<E>(&self, token: &str) -> E
    where
        E: Eq + serde::Serialize + Copy + Debug + TryInto<usize> + TryFrom<usize>,
        usize: TryFrom<E>,
    {
        let n: usize = token.parse().unwrap();
        n.try_into().unwrap_or_else(|_| panic!("Err!!!"))
        // token.parse().unwrap()
    }

    fn add<E>(&mut self, token: &str) -> E
    where
        E: Eq + serde::Serialize + Copy + Debug + TryInto<usize> + TryFrom<usize>,
        usize: TryFrom<E>,
    {
        let index = self.index(token);
        let index_usize = usize::try_from(index).unwrap_or_else(|_| panic!("Err!!!")) + 1;
        self.count = max(self.count, index_usize);
        index
    }
}

impl<E> Tokenize<E> for NullTokenIndex
where
    E: Eq + serde::Serialize + Copy + Debug + TryInto<usize> + TryFrom<usize>,
    usize: TryFrom<E>,
{
    fn tokenize(&mut self, text: &str) -> Vec<E> {
        let tokenized_text: Vec<E> = text.split_whitespace().map(|x| self.add(x)).collect();
        tokenized_text
    }

    fn build(&mut self, _text: &str) {
        // do nothing (text is already tokenized)
    }

    fn get_count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenize::{NullTokenIndex, Tokenize};

    #[test]
    fn test_build_tokenizer() {
        // let mut token_index: NullTokenIndex = NullTokenIndex::new();
        let mut token_index: Box<dyn Tokenize<u16>> = Box::new(NullTokenIndex::new());
        token_index.build("1 0 0 1");
        assert_eq!(token_index.get_count(), 0);

        let tokens = token_index.tokenize("1 0 0 1");
        assert_eq!(tokens, vec![1, 0, 0, 1]);
    }

    // #[test]
    // fn test_add() {
    //     let mut token_index: TokenIndex<u16> = TokenIndex::new();
    //     assert_eq!(token_index.add("hello"), 3);
    //     assert_eq!(token_index.add("hello"), 3);
    //     assert_eq!(token_index.add("world"), 4);
    //     assert_eq!(token_index.add("hello"), 3);
    //     assert_eq!(token_index.index("hello"), 3);
    //     assert_eq!(token_index.token(3), "hello");
    //     assert_eq!(token_index.index("world"), 4);
    //     assert_eq!(token_index.token(4), "world");
    //     assert_eq!(token_index.token(0), "<unk>");
    //     assert_eq!(token_index.token(342), "<unk>");
    //     assert_eq!(token_index.index("<unk>"), 0);
    //     assert_eq!(token_index.index("universe"), 0);
    // }

    // #[test]
    // fn test_tokenize_fn() {
    //     let mut token_index: TokenIndex<u16> = TokenIndex::new();
    //     token_index.build("");
    //     let tokens = token_index.tokenize("hello world");
    //     assert_eq!(
    //         tokens,
    //         "<unk> <unk>"
    //             .split_whitespace()
    //             .map(|x| token_index.index(x))
    //             .collect::<Vec<u16>>()
    //     );

    //     token_index.build("hello wolrd");
    //     let tokens = token_index.tokenize("hello world");
    //     assert_eq!(
    //         tokens,
    //         "hello world"
    //             .split_whitespace()
    //             .map(|x| token_index.index(x))
    //             .collect::<Vec<u16>>()
    //     );
    // }
}
