use regex::{Captures, Regex};

use crate::format::Format;

mod long;
mod short;

mod parts;
use parts::*;

pub use long::LongParser;
pub use short::ShortParser;

pub trait Parse {
    fn extract(&mut self, input: &str, w: &mut (dyn std::io::Write)) -> std::io::Result<()>;
}
