use super::Text;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Span {
    pub column_start: usize,
    pub line_start: usize,
    pub file_name: String,
    pub text: Vec<Text>,
}

impl Span {
    // TODO `as_hyperlink` (using OSC8 e.g. \x1b]8;;<link>;;\a\n)
    // TODO _actually_ normalize 'file_name' (should we handle UNCs?) (we can with https://docs.rs/dunce/latest/dunce/)
    // TODO return this in 2 parts so the 'file' and the 'position' can be handled separately
    pub fn as_location(&self) -> (String, String) {
        let file = self.file_name.replace('\\', "/");
        let pos = format!(
            ":{line}:{col}",
            line = self.line_start,
            col = self.column_start
        );
        (file, pos)
    }

    // TODO this is missing some of `help` text
    //
    // we're getting:
    //     help: try
    //     |
    // 112 -             Some((level, tail)) if tail.is_empty() => Err(Self::Err::raw(
    // 112 +             Some((level, "")) => Err(Self::Err::raw(
    //     |
    //
    // but we're formatting:
    //   Some((level, tail)) if tail.is_empty() => Err(Self::Err::raw(
    //
    pub fn partition_explain(&self) -> impl Iterator<Item = (&str, &str, &str)> + '_ {
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
                    left_pad = span.text.len() - s.len();
                }

                let start = span.highlight_start.saturating_sub(left_pad + 1);
                let end = span.highlight_end.saturating_sub(left_pad + 1);

                let start = str_indices::chars::from_byte_idx(&span.text, start);
                let end = str_indices::chars::from_byte_idx(&span.text, end);

                let text = &span.text[left_pad..];
                let start = floor_char_boundary(text, start);
                let end = ceil_char_boundary(text, end);

                let head = &text[..start];
                let mid = &text[start..end];
                let tail = &text[end..];

                // error messages are 1 indexed
                break Some((head, mid, tail));
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
