use std::rc::Rc;

pub mod buf_reader;
pub mod txt_reader;

pub type DataReader = Iterator<Item = (usize, Rc<String>)>;

pub use self::txt_reader::TxtReader;
