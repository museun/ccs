use std::{borrow::Cow, collections::HashMap, str::FromStr};

use owo_colors::DynColors;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub warning: Color,
    pub error: Color,
    pub note: Color,
    pub unknown: Color,
    pub message: Color,
    pub code: Color,
    pub highlight: Color,
    pub continuation: Color,
    pub location: Color,
    pub lint_name: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    pub const DEFAULT: Self = Self {
        warning: Color::BRIGHT_YELLOW,
        error: Color::BRIGHT_RED,
        note: Color::BRIGHT_GREEN,
        unknown: Color::CYAN,
        code: Color::WHITE,
        highlight: Color::BRIGHT_BLUE,
        lint_name: Color::MAGENTA,
        location: Color::BRIGHT_BLACK,
        message: Color::BRIGHT_WHITE,
        continuation: Color::BRIGHT_BLACK,
    };

    pub fn load(mut map: HashMap<String, String>) -> Option<(Self, Report)> {
        fn try_get(
            report: &mut Report,
            map: &mut HashMap<String, String>,
            key: &str,
            mut set: impl FnMut(Color),
        ) {
            match map.remove(key).map(|c| c.parse::<Color>()) {
                Some(Ok(color)) => set(color),
                Some(Err(err)) => report.invalid_values.push((key.to_string(), err)),
                _ => {}
            }
        }

        let mut report = Report::default();
        let mut this = Self::default();

        macro_rules! get {
            ($($ident:ident)*) => {
                const KNOWN_KEYS: &[&'static str] = &[$(stringify!($ident)),*];
                $(
                    try_get(&mut report, &mut map, stringify!($ident), |color| {
                        this.$ident = color
                    });
                )*
            };
        }

        get! {
            warning
            error
            unknown
            highlight
            lint_name
            location
            message
            continuation
        }

        for key in map.into_keys() {
            if !KNOWN_KEYS.contains(&&*key) {
                report.unknown_keys.push(key.to_string());
            }
        }

        Some((this, report))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl owo_colors::DynColor for Color {
    #[inline]
    fn fmt_ansi_fg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        DynColors::from(*self).fmt_ansi_fg(f)
    }

    #[inline]
    fn fmt_ansi_bg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        DynColors::from(*self).fmt_ansi_bg(f)
    }

    #[inline]
    fn fmt_raw_ansi_fg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        DynColors::from(*self).fmt_raw_ansi_fg(f)
    }

    #[inline]
    fn fmt_raw_ansi_bg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        DynColors::from(*self).fmt_raw_ansi_bg(f)
    }

    #[inline]
    fn get_dyncolors_fg(&self) -> DynColors {
        DynColors::from(*self).get_dyncolors_fg()
    }

    #[inline]
    fn get_dyncolors_bg(&self) -> DynColors {
        DynColors::from(*self).get_dyncolors_bg()
    }
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        <Cow<'_, str>>::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.rgb_to_named()
            .map_or_else(|| Cow::from(self.to_string()), Cow::from)
            .serialize(serializer)
    }
}

macro_rules! named_color {
    ($($s:expr => $ident:ident: ($r:expr, $g:expr, $b:expr))*) => {
        $( const $ident: Color = Color($r, $g, $b); )*
        const NAMED_COLORS: &'static [(&'static str, Color)] = &[$(($s, Self::$ident)),*];
    };
}

impl Color {
    named_color! {
        "Black"         => BLACK          : (0x40, 0x40, 0x40)
        "White"         => WHITE          : (0xC0, 0xC0, 0xC0)
        "Red"           => RED            : (0x80, 0x00, 0x00)
        "Green"         => GREEN          : (0x00, 0x80, 0x00)
        "Blue"          => BLUE           : (0x00, 0x00, 0x80)
        "Yellow"        => YELLOW         : (0x80, 0x80, 0x00)
        "Magenta"       => MAGENTA        : (0x80, 0x00, 0x80)
        "Cyan"          => CYAN           : (0x00, 0x80, 0x80)
        "BrightBlack"   => BRIGHT_BLACK   : (0x80, 0x80, 0x80)
        "BrightWhite"   => BRIGHT_WHITE   : (0xFF, 0xFF, 0xFF)
        "BrightRed"     => BRIGHT_RED     : (0xFF, 0x00, 0x00)
        "BrightGreen"   => BRIGHT_GREEN   : (0x00, 0xFF, 0x00)
        "BrightBlue"    => BRIGHT_BLUE    : (0x00, 0x00, 0xFF)
        "BrightYellow"  => BRIGHT_YELLOW  : (0xFF, 0xFF, 0x00)
        "BrightMagenta" => BRIGHT_MAGENTA : (0xFF, 0x00, 0xFF)
        "BrightCyan"    => BRIGHT_CYAN    : (0x00, 0xFF, 0xFF)
    }
}

impl Color {
    fn rgb_to_named(self) -> Option<&'static str> {
        Self::NAMED_COLORS
            .iter()
            .find_map(|&(name, color)| (color == self).then_some(name))
    }

    fn try_from_named(input: &str) -> Result<Self, String> {
        fn cmp(l: &str, r: &str) -> bool {
            if l.starts_with("bright") || l.starts_with("Bright") {
                let mut l = l.to_string();
                l.retain(|c| !c.is_ascii_whitespace());
                return l.eq_ignore_ascii_case(r);
            }

            l.eq_ignore_ascii_case(r)
        }

        for (named, color) in Self::NAMED_COLORS {
            if cmp(input, named) {
                return Ok(*color);
            }
        }

        Err(format!("unknown color: {input}"))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b) = self;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn is_hex_like(s: &str) -> bool {
            s.chars()
                .all(|c| matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9'))
        }

        let input = match input.len() {
            7 if is_hex_like(&input[1..]) => &input[1..],
            6 if is_hex_like(input) => input,
            _ => return Self::try_from_named(input),
        };

        let color =
            u32::from_str_radix(input, 16).map_err(|_| String::from("invalid hex digit"))?;

        let (r, g, b) = (
            ((color >> 16) & 0xFF) as _,
            ((color >> 8) & 0xFF) as _,
            (color & 0xFF) as _,
        );

        Ok(Self(r, g, b))
    }
}

impl From<Color> for DynColors {
    fn from(Color(r, g, b): Color) -> Self {
        Self::Rgb(r, g, b)
    }
}

#[derive(Default, Debug)]
pub struct Report {
    pub unknown_keys: Vec<String>,
    pub invalid_values: Vec<(String, String)>,
}
