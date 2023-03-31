use owo_colors::OwoColorize as _;

use crate::{Render, Theme};

use super::Text;

#[derive(Debug, serde::Deserialize)]
pub struct Span {
    pub column_start: usize,
    pub line_start: usize,
    pub file_name: String,
    pub text: Vec<Text>,
}

impl Span {
    pub(super) fn render(
        &self,
        render: Render,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if matches!(render, Render::Full) {
            use owo_colors::OwoColorize as _;
            self.relocate().try_for_each(|(start, end, text)| {
                let head = &text[..start];
                let mid = &text[start..end];
                let tail = &text[end..];
                writeln!(out, "  {head}{mid}{tail}", mid = mid.color(theme.highlight))
            })?;
        }

        const CONTINUATION: char = '⮡';

        let location = format!(
            "{file}:{line}:{col}",
            file = self.file_name,
            line = self.line_start,
            col = self.column_start,
        );

        write!(
            out,
            " {cont} {location} ",
            cont = CONTINUATION.color(theme.continuation),
            location = location.color(theme.location)
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
