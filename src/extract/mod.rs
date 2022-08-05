use regex::{Captures, Regex};

use crate::format::Format;

mod long;
mod short;

mod parts;
use parts::*;

pub use long::Long;
pub use short::Short;

pub trait Extract {
    fn extract(&mut self, input: &str, w: &mut (dyn std::io::Write)) -> std::io::Result<()>;
}
