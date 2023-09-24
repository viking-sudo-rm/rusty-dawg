pub trait VecBacking<T, TRef, TMutRef> {
    fn new() -> Self
    where
        Self: Sized;

    fn with_capacity(size: usize) -> Self
    where
        Self: Sized;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn index(&self, index: usize) -> TRef;

    fn index_mut(&mut self, index: usize) -> TMutRef;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
