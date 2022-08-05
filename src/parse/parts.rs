use regex::Captures;

use super::Format;

#[derive(Debug)]
pub struct Message(pub String);

impl Format for Message {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(w, "{}", yansi::Paint::white(&self.0).bold())
    }
}

#[derive(Debug)]
pub struct Path(pub String);

impl Format for Path {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(
            w,
            " {} {}",
            yansi::Paint::white("⮡").dimmed(),
            yansi::Paint::cyan(&self.0).dimmed()
        )
    }
}

#[derive(Debug)]
pub enum Note {
    Note(String),
    Help(String),
}

impl Format for Note {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Note(msg) => writeln!(w, "{}", yansi::Paint::white(msg).dimmed()),
            Self::Help(msg) => writeln!(w, "{}", yansi::Paint::green(msg)),
        }
    }
}

impl Note {
    pub fn extract(caps: &Captures<'_>) -> Option<Self> {
        let ctor = match caps.name("note")?.as_str() {
            s if s == "note" => Note::Note,
            s if s == "help" => Note::Help,
            _ => return None,
        };
        Some(ctor(caps.name("help")?.as_str().to_string()))
    }
}

#[derive(Debug)]
pub enum LintKind {
    Error(Option<String>),
    Warning,
}

impl Format for LintKind {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        use yansi::Paint;
        match self {
            LintKind::Error(Some(code)) => write!(w, "{} ", Paint::red(code)),
            LintKind::Error(None) => write!(w, "{} ", Paint::red("error")),
            LintKind::Warning => write!(w, "{} ", Paint::yellow("warning")),
        }
    }
}

impl LintKind {
    pub fn extract(caps: &Captures<'_>) -> Option<Self> {
        Some(match caps.name("kind")?.as_str() {
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
pub struct Context(pub String);

impl Format for Context {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(w, "--> {}", self.0)
    }
}
