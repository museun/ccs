// #![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use std::{
    borrow::Cow,
    ffi::OsStr,
    io::{stdout, BufRead},
    process::Stdio,
};

use anyhow::Context;
use gumdrop::Options;
use once_cell::sync::Lazy;
use regex::Regex;

static PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(?P<path>^.*?:\d{1,}:\d{1,}):\s(?P<kind>(error\[(?P<code>E\d{1,})\]|warning)):\s(?P<message>.*?)$"#)
        .unwrap()
});

#[derive(Debug)]
struct Command<'a> {
    args: Vec<Cow<'a, OsStr>>,
}

impl<'a> Command<'a> {
    fn annoying() -> Self {
        fn s(s: &str) -> Cow<'_, OsStr> {
            Cow::Borrowed(OsStr::new(s))
        }

        Self {
            args: vec![s("-W"), s("clippy::all"), s("-W"), s("clippy::nursery")],
        }
    }

    const fn clippy() -> Self {
        Self { args: vec![] }
    }

    const fn as_command(&self) -> &'static str {
        "clippy"
    }

    fn find_cargo(toolchain: Toolchain) -> Option<String> {
        let mut cmd = std::process::Command::new("rustup");
        if let Some(toolchain) = toolchain.as_str() {
            cmd.arg(toolchain);
        }

        let mut output = cmd
            .args(["which", "cargo"])
            .output()
            .ok()
            .map(|c| String::from_utf8(c.stdout))?
            .ok()?;

        output.drain(output.trim_end().len()..);
        Some(output)
    }

    fn build_command(self, extra: Vec<String>, toolchain: Toolchain) -> anyhow::Result<Vec<u8>> {
        const SHORT: &str = "--message-format=short";

        let cargo = Self::find_cargo(toolchain).with_context(|| "cannot find cargo via rustup")?;
        let mut cmd = std::process::Command::new(&cargo);
        cmd.stdout(Stdio::piped());

        cmd.args([self.as_command(), SHORT]);

        let mut sep = false;
        match &*self.args {
            [args @ ..] if !args.is_empty() => {
                cmd.arg("--");
                cmd.args(args);
                sep = true;
            }
            _ => {}
        }

        if !extra.is_empty() {
            if !sep {
                cmd.arg("--");
            }
            cmd.args(extra);
        }

        Ok(cmd.output()?.stderr)
    }
}

#[derive(Default, Copy, Clone, Debug)]
enum Toolchain {
    #[default]
    Stable,
    Nightly,
}

impl Toolchain {
    const fn as_str(self) -> Option<&'static str> {
        if let Self::Nightly = self {
            return Some("+nightly");
        }
        None
    }
}

enum LintKind {
    Error(String),
    Warning,
}

struct Line {
    path: String,
    kind: LintKind,
    message: String,
}

impl Line {
    fn extract(re: &Regex, input: &str) -> Option<Self> {
        let caps = re.captures(input)?;
        let path = caps.name("path")?.as_str();

        let kind = match caps.name("kind")?.as_str() {
            s if s.starts_with("error") => LintKind::Error(caps.name("code")?.as_str().to_string()),
            "warning" => LintKind::Warning,
            s => unreachable!("unknown: {s}"),
        };

        let message = caps.name("message")?.as_str();

        Some(Self {
            path: path.to_string(),
            kind,
            message: message.to_string(),
        })
    }
}

struct State<'a> {
    re: &'a Regex,
    line: usize,
    line_breaks: bool,
}

impl<T> WriteExt for T where T: std::io::Write {}
trait WriteExt: std::io::Write {
    fn format_line(&mut self, line: &str, state: &mut State) -> std::io::Result<()> {
        use yansi::Paint;

        let line = match Line::extract(state.re, line) {
            Some(line) => line,
            None => return Ok(()),
        };

        if state.line_breaks && state.line > 0 {
            writeln!(self)?;
        }

        match line.kind {
            LintKind::Error(code) => write!(self, "{} ", Paint::red(code)),
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

#[derive(gumdrop::Options, Debug)]
struct Args {
    #[options(help_flag)]
    help: bool,

    #[options(help = "use nightly", default = "false")]
    nightly: bool,

    #[options(help = "use line breaks", default = "false")]
    line_breaks: bool,

    #[options(
        short = "w",
        long = "warn",
        help = "additional warning levels to use",
        meta = "string"
    )]
    additional: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse_args_default_or_exit();

    // TODO disable colors via flag
    if std::env::var("NO_COLOR").is_ok() {
        yansi::Paint::disable()
    }

    let command = args
        .nightly
        .then(Command::annoying)
        .unwrap_or_else(Command::clippy);

    let toolchain = args
        .nightly
        .then_some(Toolchain::Nightly)
        .unwrap_or_default();

    let child = command.build_command(args.additional, toolchain)?;

    let mut w = stdout();
    let mut state = State {
        re: &PATTERN,
        line: 0,
        line_breaks: args.line_breaks,
    };

    for line in std::io::BufReader::new(&*child).lines().flatten() {
        w.format_line(&line, &mut state)?;
    }

    Ok(())
}
