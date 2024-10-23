use std::{borrow::Cow, ffi::OsStr, io::Read, path::PathBuf, process::Stdio};

use anyhow::Context;

use crate::Tool;

#[derive(Debug)]
pub struct Command<'a> {
    pub args: Vec<Cow<'a, OsStr>>,
}

fn os_str(s: &str) -> Cow<'_, OsStr> {
    Cow::Borrowed(OsStr::new(s))
}

impl<'a> Command<'a> {
    pub fn annoying() -> Self {
        Self {
            args: vec![
                os_str("-W"),
                os_str("clippy::all"),
                os_str("-W"),
                os_str("clippy::nursery"),
            ],
        }
    }

    pub fn more_annoying() -> Self {
        Self {
            args: vec![
                os_str("-W"),
                os_str("clippy::all"),
                os_str("-W"),
                os_str("clippy::nursery"),
                os_str("-W"),
                os_str("clippy::pedantic"),
            ],
        }
    }

    pub const fn default_lints() -> Self {
        Self { args: vec![] }
    }

    pub fn build_command(self, opts: Options) -> anyhow::Result<impl Read> {
        let Options {
            extra,
            path,
            toolchain,
            target,
            features,
            dry_run,
            tool,
        } = opts;

        let cargo = crate::find_cargo(toolchain).with_context(|| "cannot find cargo via rustup")?;
        let mut cmd = std::process::Command::new(&cargo);
        cmd.stdout(Stdio::piped());

        cmd.args([Self::as_command(tool), "--message-format=json"]);
        if let Some(path) = path {
            cmd.arg("--manifest-path");
            cmd.arg(path);
        }

        match target {
            Target::All => {
                cmd.arg("--all-targets");
            }
            Target::Example => {
                cmd.arg("--examples");
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

        cmd.arg("--");

        if !self.args.is_empty() {
            cmd.args(&self.args);
        }

        for (key, val) in extra.as_flags() {
            cmd.arg(key);
            cmd.arg(val);
        }

        if dry_run {
            let args =
                cmd.get_args()
                    .map(OsStr::to_string_lossy)
                    .fold(String::new(), |mut a, c| {
                        if !a.is_empty() {
                            a.push(' ');
                        }
                        a.push_str(&c);
                        a
                    });
            let name = cmd.get_program().to_string_lossy();
            println!("{name} {args}");
            std::process::exit(0);
        }

        let child = cmd.spawn()?;
        let stderr = child.stdout.expect("stdout attached to the child process");

        Ok(stderr)
    }

    const fn as_command(tool: Tool) -> &'static str {
        match tool {
            Tool::Clippy => "clippy",
            Tool::Check => "check",
        }
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
        self.allow.is_empty() && self.warning.is_empty() && self.deny.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    All,
    Test,
    Example,
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
    pub toolchain: Toolchain,
    pub target: Target,
    pub features: Features,
    pub dry_run: bool,
    pub tool: Tool,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum Toolchain {
    #[default]
    Stable,
    Nightly,
}

impl Toolchain {
    pub const fn as_str(self) -> Option<&'static str> {
        if matches!(self, Self::Nightly) {
            return Some("+nightly");
        }
        None
    }
}
