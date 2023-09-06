use std::error::Error;

pub trait Load {

    fn load(save_path: &str) -> Result<Self, Box<dyn Error>> where Self: Sized;

}