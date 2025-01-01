use std::borrow::Cow;

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
        continuation: &Option<Cow<'static, str>>,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if matches!(render_options.render, RenderStyle::Full) {
            use owo_colors::OwoColorize as _;
            self.relocate().try_for_each(|(start, end, text)| {
                let start = floor_char_boundary(text, start);
                let end = ceil_char_boundary(text, end);

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

        let location = format!(
            "\x1b]8;;{file}:{line}:{col}\x1b\\{file}:{line}:{col}\x1b]8;;\x1b\\\n",
            file = self.file_name,
            line = self.line_start,
            col = self.column_start,
        );

        match continuation {
            Some(continuation) => {
                write!(
                    out,
                    " {cont} {location} ",
                    cont = continuation.color(theme.continuation),
                    location = location.color(theme.location)
                )
            }
            None => write!(
                out,
                " {location} ",
                location = location.color(theme.location)
            ),
        }
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

                let start = span.highlight_start.saturating_sub(left_pad + 1);
                let end = span.highlight_end.saturating_sub(left_pad + 1);

                let start = str_indices::chars::from_byte_idx(&span.text, start);
                let end = str_indices::chars::from_byte_idx(&span.text, end);

                // error messages are 1 indexed
                break Some((
                    start,
                    end,
                    // TODO use unicode-segmentation here
                    // what does this mean? how would segmentation be applicable here?
                    &span.text[left_pad..],
                ));
            }
        })
    }
}

// NOTE this is taken from <https://github.com/rust-lang/rust/issues/93743>
// TODO its currently unstable but its fine for what we need
fn floor_char_boundary(str: &str, index: usize) -> usize {
    if index >= str.len() {
        str.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = str.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|&b| is_utf8_char_boundary(b));

        lower_bound + new_index.unwrap()
    }
}

// NOTE this is taken from <https://github.com/rust-lang/rust/issues/93743>
// TODO its currently unstable but its fine for what we need
fn ceil_char_boundary(str: &str, index: usize) -> usize {
    if index > str.len() {
        str.len()
    } else {
        let upper_bound = Ord::min(index + 4, str.len());
        str.as_bytes()[index..upper_bound]
            .iter()
            .position(|&b| is_utf8_char_boundary(b))
            .map_or(upper_bound, |pos| pos + index)
    }
}

// NOTE impl detail of `u8::is_utf8_char_boundary` used by `floor_char_boundary` and `ceil_char_boundary`
const fn is_utf8_char_boundary(byte: u8) -> bool {
    // This is bit magic equivalent to: b < 128 || b >= 192
    (byte as i8) >= -0x40
}
