use owo_colors::{DynColor, OwoColorize as _};

use crate::{IncludeNotes, RenderOptions, Theme};

use super::{Code, Level, Span};

// TODO handle notes
#[derive(Debug, serde::Deserialize)]
pub struct Message {
    pub code: Option<Code>,
    pub message: String,
    pub level: Level,
    pub spans: Vec<Span>,
    pub children: Vec<Self>,
}

impl Message {
    pub(super) fn render(
        &self,
        render_options: &RenderOptions,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        let color = match self.level {
            Level::Warning => theme.warning,
            Level::Error => theme.error,
            Level::Note => theme.note,
            _ => theme.unknown,
        };

        let ignored = matches!(self.level, Level::Note)
            && matches!(render_options.include_notes, IncludeNotes::No);

        if !ignored {
            self.header(color, render_options.include_notes, theme, out)?;
        }

        self.spans.iter().try_for_each(|span| {
            if ignored {
                return Ok(());
            }

            span.render(render_options, theme, out)?;
            if matches!(self.level, Level::Warning) {
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
        include_notes: IncludeNotes,
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
            Level::Note if matches!(include_notes, IncludeNotes::Yes) => {
                write!(out, "{note} ", note = "note".color(color))?;
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
