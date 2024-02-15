pub trait VecBacking<T> {
    type TRef;
    type TMutRef;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn index(&self, index: usize) -> Self::TRef;

    fn index_mut(&mut self, index: usize) -> Self::TMutRef;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
