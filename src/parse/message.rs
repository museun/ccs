use owo_colors::{DynColor, OwoColorize as _};

use crate::{Render, Theme};

use super::{Code, Level, Span};

// TODO handle notes
#[derive(Debug, serde::Deserialize)]
pub struct Message {
    pub code: Option<Code>,
    pub message: String,
    pub level: Level,
    pub spans: Vec<Span>,
}

impl Message {
    pub(super) fn render(
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
