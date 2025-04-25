// Comparator for CdawgEdgeWeights that looks them up in tokens.

use comparator::Comparator;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::cdawg::token_backing::TokenBacking;
use crate::graph::indexing::IndexType;

const END: u16 = u16::MAX;

pub struct CdawgComparator {
    tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
    token1: Option<u16>, // If token is provided, it is assumed to be the token for e1.
}

impl CdawgComparator {
    pub fn new(tokens: Rc<RefCell<dyn TokenBacking<u16>>>) -> Self {
        Self {
            tokens,
            token1: None,
        }
    }

    pub fn new_with_token(tokens: Rc<RefCell<dyn TokenBacking<u16>>>, token: u16) -> Self {
        Self {
            tokens,
            token1: Some(token),
        }
    }
}

impl<Ix> Comparator<(Ix, Ix)> for CdawgComparator
where
    Ix: IndexType,
{
    fn compare(&self, e1: &(Ix, Ix), e2: &(Ix, Ix)) -> Ordering {
        let token1 = match self.token1 {
            Some(tok) => tok,
            None => self.tokens.borrow().get(e1.0.index()),
        };
        let token2 = self.tokens.borrow().get(e2.0.index());

        if token1 == END && token2 == END {
            // The start index of an open node represents doc_id
            e1.0.cmp(&e2.0)
        } else if token1 == token2 {
            Ordering::Equal
        } else if token1 < token2 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::graph::indexing::DefaultIx;

    // Converts an integer into an index
    fn i(x: usize) -> DefaultIx {
        DefaultIx::new(x)
    }

    #[test]
    fn test_compare_no_token() {
        let tokens = Rc::new(RefCell::new(vec![2, 1, 0, 1, 2, END, END]));
        let cmp = CdawgComparator::new(tokens);

        assert_eq!(cmp.compare(&(i(0), i(5)), &(i(4), i(5))), Ordering::Equal);
        assert_eq!(cmp.compare(&(i(0), i(5)), &(i(1), i(5))), Ordering::Greater);
        assert_eq!(cmp.compare(&(i(1), i(5)), &(i(0), i(5))), Ordering::Less);

        // Now with end-of-text weights.
        assert_eq!(cmp.compare(&(i(5), i(6)), &(i(5), i(7))), Ordering::Equal);
        assert_eq!(cmp.compare(&(i(5), i(6)), &(i(6), i(7))), Ordering::Less);
        assert_eq!(cmp.compare(&(i(5), i(6)), &(i(0), i(3))), Ordering::Greater);
    }

    #[test]
    fn test_compare_token() {
        let tokens = Rc::new(RefCell::new(vec![2, 1, 0, 1, 2]));
        let cmp = CdawgComparator::new_with_token(tokens, 1);

        assert_eq!(cmp.compare(&(i(0), i(5)), &(i(0), i(5))), Ordering::Less);
        assert_eq!(cmp.compare(&(i(0), i(5)), &(i(1), i(5))), Ordering::Equal);
        assert_eq!(cmp.compare(&(i(1), i(5)), &(i(2), i(5))), Ordering::Greater);
    }

    #[test]
    fn test_compare_end() {
        let tokens = Rc::new(RefCell::new(vec![2, 1, END, 1, END]));
        let cmp = CdawgComparator::new_with_token(tokens, END);

        assert_eq!(cmp.compare(&(i(2), i(3)), &(i(4), i(5))), Ordering::Less);
        assert_eq!(cmp.compare(&(i(4), i(5)), &(i(4), i(5))), Ordering::Equal);
        assert_eq!(cmp.compare(&(i(2), i(3)), &(i(0), i(5))), Ordering::Greater);
    }
}
