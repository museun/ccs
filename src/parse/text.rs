#[derive(Debug, serde::Deserialize)]
pub struct Text {
    pub highlight_start: usize, // are these byte or grapheme indices?
    pub highlight_end: usize,   // are these byte or grapheme indices?
    pub text: String,
}
