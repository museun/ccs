use regex::{Captures, Regex};

pub trait Parse {
    type Output;
    fn extract(re: &Regex, input: &str) -> Option<Self::Output>;
}

#[derive(Debug)]
pub enum LintKind {
    Error(Option<String>),
    Warning,
}

impl LintKind {
    fn extract(caps: &Captures<'_>) -> Option<Self> {
        Some(match dbg!(caps.name("kind"))?.as_str() {
            s if s.starts_with("error") => caps
                .name("code")
                .map(|c| c.as_str().to_string())
                .map(Some)
                .map(LintKind::Error)
                .unwrap_or(LintKind::Error(None)),
            "warning" => LintKind::Warning,
            s => unreachable!("unknown: {s}"),
        })
    }
}

#[derive(Debug)]
pub struct ShortLine {
    pub path: String,
    pub kind: LintKind,
    pub message: String,
}

pub struct ShortParser;
impl ShortParser {
    pub const PATTERN: &'static str = r#"(?m)(?P<path>^.*?:\d{1,}:\d{1,}):\s(?P<kind>(error\[?(?P<code>E\d{1,})?\]?|warning)):\s(?P<message>.*?)$"#;
}

impl Parse for ShortParser {
    type Output = ShortLine;

    fn extract(re: &Regex, input: &str) -> Option<Self::Output> {
        let caps = dbg!(re.captures(input))?;
        let path = caps.name("path")?.as_str();

        let kind = LintKind::extract(&caps)?;
        let message = caps.name("message")?.as_str();

        Some(ShortLine {
            path: path.to_string(),
            kind,
            message: message.to_string(),
        })
    }
}
