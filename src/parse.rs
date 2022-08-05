use regex::Regex;

pub enum LintKind {
    Error(Option<String>),
    Warning,
}

pub struct Line {
    pub path: String,
    pub kind: LintKind,
    pub message: String,
}

impl Line {
    pub fn extract(re: &Regex, input: &str) -> Option<Self> {
        let caps = re.captures(input)?;
        let path = caps.name("path")?.as_str();

        let kind = match caps.name("kind")?.as_str() {
            s if s.starts_with("error") => caps
                .name("code")
                .map(|c| c.as_str().to_string())
                .map(Some)
                .map(LintKind::Error)
                .unwrap_or(LintKind::Error(None)),
            "warning" => LintKind::Warning,
            s => unreachable!("unknown: {s}"),
        };

        let message = caps.name("message")?.as_str();

        Some(Self {
            path: path.to_string(),
            kind,
            message: message.to_string(),
        })
    }
}
