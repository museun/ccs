#[derive(Debug, serde::Deserialize)]
pub struct Text {
    pub highlight_start: usize,
    pub highlight_end: usize,
    pub text: String,
}
