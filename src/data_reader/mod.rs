use std::rc::Rc;

mod buf_reader;
mod jsonl_reader;
mod pile_reader;
mod txt_reader;

pub type DataReader = dyn Iterator<Item = (usize, Rc<String>)>;

pub use self::jsonl_reader::JsonlReader;
pub use self::pile_reader::PileReader;
pub use self::txt_reader::TxtReader;
