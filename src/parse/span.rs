use owo_colors::OwoColorize as _;

use crate::{RenderOptions, RenderStyle, Theme};

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
        render_options: &RenderOptions,
        theme: &Theme,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if matches!(render_options.render, RenderStyle::Full) {
            use owo_colors::OwoColorize as _;
            self.relocate().try_for_each(|(start, end, text)| {
                let head = &text[..start];
                let mid = &text[start..end];
                let tail = &text[end..];
                writeln!(
                    out,
                    "  {head}{mid}{tail}",
                    head = head.color(theme.code),
                    mid = mid.color(theme.highlight),
                    tail = tail.color(theme.code)
                )
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
            loop {
                let (i, span) = iter.next()?;
                if span.text.trim_start().is_empty() {
                    continue;
                }

                if i == 0 {
                    let s = span.text.trim_start();
                    // TODO use unicode-width here
                    left_pad = span.text.len() - s.len();
                }

                // error messages are 1 indexed
                break Some((
                    span.highlight_start.saturating_sub(left_pad + 1),
                    span.highlight_end.saturating_sub(left_pad + 1),
                    // TODO use unicode-segmentation here
                    &span.text[left_pad..],
                ));
            }
        })
    }
}
