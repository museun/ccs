use super::*;

#[derive(Debug)]
pub struct ShortLine {
    pub path: Path,
    pub kind: LintKind,
    pub message: Message,
}

impl Format for ShortLine {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.kind.format(w)?;
        self.message.format(w)?;
        self.path.format(w)
    }
}

pub struct ShortParser {
    re: Regex,
}

impl ShortParser {
    pub const PATTERN: &'static str = r#"(?m)(?P<path>^.*?:\d{1,}:\d{1,}):\s(?P<kind>(error\[?(?P<code>E\d{1,})?\]?|warning)):\s(?P<message>.*?)$"#;
    pub fn new() -> Self {
        Self {
            re: Regex::new(Self::PATTERN).unwrap(),
        }
    }
}

impl Parse for ShortParser {
    fn extract(&mut self, input: &str, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        macro_rules! maybe {
            ($expr:expr) => {
                match $expr {
                    Some(d) => d,
                    None => return Ok(()),
                }
            };
        }

        let caps = maybe!(self.re.captures(input));
        let path = maybe!(caps.name("path")).as_str();

        let kind = maybe!(LintKind::extract(&caps));
        let message = maybe!(caps.name("message")).as_str();

        let line = ShortLine {
            path: Path(path.to_string()),
            kind,
            message: Message(message.to_string()),
        };

        line.format(w)
    }
}
