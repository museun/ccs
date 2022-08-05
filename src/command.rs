use std::{borrow::Cow, ffi::OsStr, io::Read, path::PathBuf, process::Stdio};

use anyhow::Context;

#[derive(Debug)]
pub struct Options {
    pub extra: Vec<String>,
    pub path: Option<PathBuf>,
    pub format: Format,
    pub toolchain: Toolchain,
    pub tests: bool,
}

#[derive(Debug)]
pub struct Command<'a> {
    pub args: Vec<Cow<'a, OsStr>>,
}

impl<'a> Command<'a> {
    pub fn annoying() -> Self {
        fn s(s: &str) -> Cow<'_, OsStr> {
            Cow::Borrowed(OsStr::new(s))
        }

        Self {
            args: vec![s("-W"), s("clippy::all"), s("-W"), s("clippy::nursery")],
        }
    }

    pub const fn clippy() -> Self {
        Self { args: vec![] }
    }

    pub fn build_command(self, opts: Options) -> anyhow::Result<impl Read> {
        let Options {
            extra,
            path,
            format,
            toolchain,
            tests,
        } = opts;

        let cargo = Self::find_cargo(toolchain).with_context(|| "cannot find cargo via rustup")?;
        let mut cmd = std::process::Command::new(&cargo);
        cmd.stderr(Stdio::piped());

        cmd.args([self.as_command(), format.as_str()]);
        if let Some(path) = path {
            cmd.arg("--manifest-path");
            cmd.arg(path);
        }

        if tests {
            cmd.arg("--tests");
        }

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
            for extra in extra {
                cmd.arg("-W");
                cmd.arg(extra);
            }
        }

        let child = cmd.spawn()?;
        let stderr = child.stderr.expect("stderr attached to the child process");

        Ok(stderr)
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
}

#[derive(Default, Copy, Clone, Debug)]
pub enum Toolchain {
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

#[derive(Default, Copy, Clone, Debug)]
pub enum Format {
    Human,
    #[default]
    Short,
}

impl Format {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Human => "--message-format=human",
            Self::Short => "--message-format=short",
        }
    }
}
