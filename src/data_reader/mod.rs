use std::rc::Rc;

mod buf_reader;
pub mod pile_reader;
pub mod txt_reader;

pub type DataReader = dyn Iterator<Item = (usize, Rc<String>)>;

pub use self::pile_reader::PileReader;
pub use self::txt_reader::TxtReader;
