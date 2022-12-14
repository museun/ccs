use std::{borrow::Cow, ffi::OsStr, io::Read, path::PathBuf, process::Stdio};

use anyhow::Context;

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
            target,
            features,
            dry_run,
        } = opts;

        let cargo = crate::find_cargo(toolchain).with_context(|| "cannot find cargo via rustup")?;
        let mut cmd = std::process::Command::new(&cargo);
        cmd.stderr(Stdio::piped());

        cmd.args([self.as_command(), format.as_str()]);
        if let Some(path) = path {
            cmd.arg("--manifest-path");
            cmd.arg(path);
        }

        match target {
            Target::All => {
                cmd.arg("--all-targets");
            }
            Target::Test => {
                cmd.arg("--tests");
            }
            Target::Specific(targets) => {
                for target in targets {
                    cmd.arg("--target").arg(target);
                }
            }

            Target::Default => {}
        }

        match features {
            Features::All => {
                cmd.arg("--all-features");
            }
            Features::Specific(features) => {
                for feature in features {
                    cmd.arg("--features").arg(feature);
                }
            }

            Features::Default => {}
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

        if !extra.is_empty() && !sep {
            cmd.arg("--");
        }

        for (key, val) in extra.as_flags() {
            cmd.arg(key);
            cmd.arg(val);
        }

        if dry_run {
            let args =
                cmd.get_args()
                    .map(|c| c.to_string_lossy())
                    .fold(String::new(), |mut a, c| {
                        if !a.is_empty() {
                            a.push(' ');
                        }
                        a.push_str(&*c);
                        a
                    });
            let name = cmd.get_program().to_string_lossy();
            println!("{name} {args}");
            std::process::exit(0);
        }

        let child = cmd.spawn()?;
        let stderr = child.stderr.expect("stderr attached to the child process");

        Ok(stderr)
    }

    const fn as_command(&self) -> &'static str {
        "clippy"
    }
}

#[derive(Debug, Default)]
pub struct Extra {
    pub allow: Vec<String>,
    pub warning: Vec<String>,
    pub deny: Vec<String>,
}

impl Extra {
    pub fn as_flags(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
        std::iter::repeat("-A")
            .zip(self.allow.iter().map(|s| &**s))
            .chain(std::iter::repeat("-W").zip(self.warning.iter().map(|s| &**s)))
            .chain(std::iter::repeat("-D").zip(self.deny.iter().map(|s| &**s)))
    }

    pub fn is_empty(&self) -> bool {
        !self.allow.is_empty() || !self.warning.is_empty() || !self.deny.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    All,
    Test,
    Default,
    Specific(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Features {
    All,
    Specific(Vec<String>),
    Default,
}

#[derive(Debug)]
pub struct Options {
    pub extra: Extra,
    pub path: Option<PathBuf>,
    pub format: OutputKind,
    pub toolchain: Toolchain,
    pub target: Target,
    pub features: Features,
    pub dry_run: bool,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum Toolchain {
    #[default]
    Stable,
    Nightly,
}

impl Toolchain {
    pub const fn as_str(self) -> Option<&'static str> {
        if let Self::Nightly = self {
            return Some("+nightly");
        }
        None
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub enum OutputKind {
    Human,
    #[default]
    Short,
}

impl OutputKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Human => "--message-format=human",
            Self::Short => "--message-format=short",
        }
    }
}
