use regex::Regex;
use yansi::Paint;

use crate::parse::{LintKind, Parse, ShortParser};

pub struct State<'a> {
    re: &'a Regex,
    line: usize,
    line_breaks: bool,
}

impl<'a> State<'a> {
    pub const fn new(re: &'a Regex, line_breaks: bool) -> Self {
        Self {
            re,
            line: 0,
            line_breaks,
        }
    }
}

pub trait WriteExt: std::io::Write {
    fn format_line(&mut self, line: &str, state: &mut State) -> std::io::Result<()> {
        let line = match ShortParser::extract(state.re, line) {
            Some(line) => line,
            None => return Ok(()),
        };

        if state.line_breaks && state.line > 0 {
            writeln!(self)?;
        }

        match line.kind {
            LintKind::Error(Some(code)) => write!(self, "{} ", Paint::red(code)),
            LintKind::Error(None) => write!(self, "{} ", Paint::red("error")),
            LintKind::Warning => write!(self, "{} ", Paint::yellow("warning")),
        }?;

        writeln!(self, "{}", Paint::white(line.message).bold())?;
        writeln!(
            self,
            " {} {}",
            Paint::white("⮡").dimmed(),
            Paint::cyan(line.path).dimmed()
        )?;

        state.line += 1;
        Ok(())
    }
}

impl<T> WriteExt for T where T: std::io::Write {}
