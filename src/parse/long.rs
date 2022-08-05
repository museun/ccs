use super::*;

pub struct LongParser {
    re: Regex,
    state: LongParserState,
}

impl LongParser {
    const PATTERN: &'static str = r#"(?x)
    ^(?:(?P<kind>(error\[?(?P<code>E\d{1,})?\]?|warning)):\s(?P<message>.*?))$ # error / warning
    |
    ^(:?\s*-->\s(?P<path>.*?))$                                                # path
    |
    ^(:?\d{1,}\s\|\s*(?P<context>.*?))$                                        # context
    |
    ^(:?(?:\s*=\s)?(?P<note>note|help):\s(?P<help>.*?))$                       # note/help messages
    "#;

    pub fn new() -> Self {
        Self {
            re: Regex::new(Self::PATTERN).unwrap(),
            state: <_>::default(),
        }
    }
}

impl Parse for LongParser {
    fn extract(&mut self, input: &str, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        fn update(caps: &Captures<'_>, path: &str, opt: &mut Option<String>) {
            if let Some(key) = caps.name(path) {
                opt.replace(key.as_str().to_string());
            }
        }

        if input.is_empty() {
            macro_rules! maybe {
                ($expr:expr) => {
                    match $expr {
                        Some(d) => d,
                        None => return Ok(()),
                    }
                };
            }

            let state = std::mem::take(&mut self.state);
            let line = LongLine {
                path: maybe!(state.path.map(Path)),
                kind: maybe!(state.kind),
                message: maybe!(state.message.map(Message)),
                context: maybe!(state.context.map(Context)),
                notes: state.notes,
            };
            return line.format(w);
        }

        let caps = match self.re.captures(input) {
            Some(caps) => caps,
            _ => return Ok(()),
        };

        if let Some(k) = LintKind::extract(&caps) {
            self.state.kind.replace(k);
        }

        update(&caps, "message", &mut self.state.message);
        update(&caps, "path", &mut self.state.path);
        update(&caps, "context", &mut self.state.context);

        self.state.notes.extend(Note::extract(&caps));
        Ok(())
    }
}

#[derive(Debug)]
struct LongLine {
    path: Path,
    kind: LintKind,
    message: Message,
    context: Context,
    #[allow(dead_code)]
    notes: Vec<Note>,
}

impl Format for LongLine {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.kind.format(w)?;
        self.message.format(w)?;
        self.context.format(w)?;

        // TODO flag for this
        // for note in &self.notes {
        //     note.format(w)?;
        // }

        self.path.format(w)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
struct LongParserState {
    path: Option<String>,
    kind: Option<LintKind>,
    message: Option<String>,
    context: Option<String>,
    notes: Vec<Note>,
}
