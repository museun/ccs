use owo_colors::{AnsiColors, DynColor, DynColors, OwoColorize};

pub fn gather_reasons(output: impl std::io::Read) -> Vec<Reason> {
    serde_json::Deserializer::from_reader(output)
        .into_iter()
        .flatten()
        .filter(Reason::is_not_empty)
        .collect::<_>()
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum Reason {
    CompilerMessage {
        message: Message,
    },
    BuildFinished {
        success: bool,
    },
    #[serde(other)]
    Ignored,
}

impl Reason {
    pub fn render(
        &self,
        render: Render,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        match self {
            Self::CompilerMessage { message } => message.render(render, theme, out),
            Self::BuildFinished { success: true } => {
                // TODO perhaps report this with a flag
                Ok(())
            }
            Self::BuildFinished { success: false } => {
                // TODO perhaps report this with a flag
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        matches!(self, Self::Ignored | Self::BuildFinished { .. })
            || matches!(self, Self::CompilerMessage{ message: Message{ spans, .. } } if spans.is_empty())
    }

    #[inline]
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}

// TODO handle notes
#[derive(Debug, serde::Deserialize)]
pub struct Message {
    pub code: Option<Code>,
    pub message: String,
    pub level: Level,
    pub spans: Vec<Span>,
}

impl Message {
    fn render(
        &self,
        render: Render,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        let color = match self.level {
            Level::Warning => theme.warning,
            Level::Error => theme.error,
            _ => theme.unknown,
        };

        self.header(color, theme, out)?;

        self.spans.iter().try_for_each(|span| {
            span.render(render, theme, out)?;
            if let Level::Warning = self.level {
                if let Some(Code { code }) = self.code.as_ref() {
                    write!(out, "({code})", code = code.color(theme.lint_name))?;
                }
            }
            writeln!(out)
        })
    }

    fn header(
        &self,
        color: impl DynColor,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        match self.level {
            Level::Error if self.code.is_some() => {
                write!(
                    out,
                    "{error} ",
                    error = self.code.as_ref().map(|c| &c.code).unwrap().color(color)
                )?;
            }
            Level::Error => {
                write!(out, "{error} ", error = "error".color(color))?;
            }
            Level::Warning => {
                write!(out, "{warning} ", warning = "warning".color(color))?;
            }
            _ => {}
        }

        writeln!(
            out,
            "{message}",
            message = self.message.color(theme.message).bold()
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Warning,
    Error,
    FailureNote,
    Help,
    Note,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, serde::Deserialize)]
pub struct Code {
    pub code: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Span {
    pub column_start: usize,
    pub line_start: usize,
    pub file_name: String,
    pub text: Vec<Text>,
}

impl Span {
    fn render(
        &self,
        render: Render,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if let Render::Full = render {
            use owo_colors::OwoColorize as _;
            self.relocate().try_for_each(|(start, end, text)| {
                let head = &text[..start];
                let mid = &text[start..end];
                let tail = &text[end..];
                writeln!(out, "  {head}{mid}{tail}", mid = mid.color(theme.highlight))
            })?;
        }

        const CONTINUATION: char = '⮡';

        write!(
            out,
            " {cont} {file}:{line}:{col} ",
            cont = CONTINUATION.color(theme.continuation),
            file = self.file_name.color(theme.location),
            line = self.line_start,
            col = self.column_start,
        )
    }

    fn relocate(&self) -> impl Iterator<Item = (usize, usize, &str)> + '_ {
        let mut iter = self.text.iter().enumerate();
        let mut left_pad = 0;
        std::iter::from_fn(move || {
            let (i, span) = iter.next()?;
            if i == 0 {
                let s = span.text.trim_start();
                left_pad = span.text.len() - s.len();
            }

            // error messages are 1 indexed
            Some((
                span.highlight_start.saturating_sub(left_pad + 1),
                span.highlight_end.saturating_sub(left_pad + 1),
                &span.text[left_pad..],
            ))
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Text {
    pub highlight_start: usize,
    pub highlight_end: usize,
    pub text: String,
}

#[derive(Default, Copy, Clone)]
pub enum Render {
    #[default]
    Short,
    Full,
}

pub struct Theme {
    pub warning: DynColors,
    pub error: DynColors,
    pub unknown: DynColors,
    pub highlight: DynColors,
    pub lint_name: DynColors,
    pub location: DynColors,
    pub message: DynColors,
    pub continuation: DynColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            warning: DynColors::Ansi(AnsiColors::BrightYellow),
            error: DynColors::Ansi(AnsiColors::BrightRed),
            unknown: DynColors::Ansi(AnsiColors::Cyan),
            highlight: DynColors::Ansi(AnsiColors::BrightBlue),
            lint_name: DynColors::Ansi(AnsiColors::Magenta),
            location: DynColors::Ansi(AnsiColors::Green),
            message: DynColors::Ansi(AnsiColors::BrightWhite),
            continuation: DynColors::Ansi(AnsiColors::BrightBlack),
        }
    }
}
