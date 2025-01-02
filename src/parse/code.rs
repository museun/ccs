#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Code {
    pub code: String,
}

impl Ord for Code {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        lower_clippy_over_rustc(&self.code, &other.code)
    }
}

impl PartialOrd for Code {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(lower_clippy_over_rustc(&self.code, &other.code))
    }
}

fn lower_clippy_over_rustc(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.starts_with("clippy::"), right.starts_with("clippy::")) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) | (true, true) => left.cmp(right),
    }
}
