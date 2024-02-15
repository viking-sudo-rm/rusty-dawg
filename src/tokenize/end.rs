// Basic trait for types that have EOT token.

pub trait End {
    fn end() -> Self;
}

impl End for u16 {
    fn end() -> Self {
        u16::MAX
    }
}

impl End for u32 {
    fn end() -> Self {
        u32::MAX
    }
}

impl End for usize {
    fn end() -> Self {
        usize::MAX
    }
}
