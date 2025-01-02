use std::{borrow::Cow, collections::HashSet, str::FromStr};

use crate::parse::Level;

#[derive(Default, Clone, Debug)]
pub struct RenderOptions {
    pub render: RenderStyle,
    pub include_notes: IncludeNotes,
    pub filter: HashSet<Filter>,
    pub continuation: Option<Cow<'static, str>>,
    pub delimiter: Option<String>,
    pub new_line: bool,
}

impl RenderOptions {
    pub fn without_error(mut self, name: impl ToString) -> Self {
        self.filter.insert(Filter::Error(name.to_string()));
        self
    }

    pub fn without_warning(mut self, name: impl ToString) -> Self {
        self.filter.insert(Filter::Warning(name.to_string()));
        self
    }

    pub fn without_errors(mut self) -> Self {
        self.filter.insert(Filter::AllErrors);
        self
    }

    pub fn without_warnings(mut self) -> Self {
        self.filter.insert(Filter::AllWarnings);
        self
    }

    pub fn is_ignored(&self, level: Level, name: Option<&str>) -> bool {
        self.filter.iter().any(|f| match f {
            Filter::Error(lint) if matches!(level, Level::Error) => {
                if let Some(name) = name {
                    lint.eq_ignore_ascii_case(name)
                } else {
                    false
                }
            }
            Filter::Warning(lint) if matches!(level, Level::Warning) => {
                if let Some(name) = name {
                    lint.eq_ignore_ascii_case(name)
                } else {
                    false
                }
            }
            Filter::AllErrors if matches!(level, Level::Error) => true,
            Filter::AllWarnings if matches!(level, Level::Warning) => true,
            _ => false,
        })
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub enum RenderStyle {
    #[default]
    Short,
    Full,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum IncludeNotes {
    Yes,
    #[default]
    No,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Filter {
    AllWarnings,
    AllErrors,
    Error(String),
    Warning(String),
}

impl FromStr for Filter {
    type Err = clap::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Self::Err::raw(
                clap::error::ErrorKind::ValueValidation,
                "input cannot be empty",
            ));
        }

        match input.split_once('=') {
            Some(("warning", "all")) => Ok(Self::AllWarnings),
            Some(("error", "all")) => Ok(Self::AllErrors),
            Some(("warning", tail)) if !tail.is_empty() => Ok(Self::Warning(tail.to_string())),
            Some(("error", tail)) if !tail.is_empty() => Ok(Self::Error(tail.to_string())),

            Some((level, tail)) if !level.is_empty() && !tail.is_empty() => Err(Self::Err::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("unknown lint/level: {level}={tail}"),
            )),

            Some((level, tail)) if level.is_empty() => Err(Self::Err::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("\n{{level}}={tail}: level cannot be empty"),
            )),
            Some((level, tail)) if tail.is_empty() => Err(Self::Err::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("\n{level}={{lint}}: lint cannot be empty"),
            )),

            _ => Err(Self::Err::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("\nexpected one of: warning | error.\ngot a lint: {input}"),
            )),
        }
    }
}
