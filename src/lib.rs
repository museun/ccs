mod args;
pub use args::Args;

mod command;
pub use command::{Command, Extra, Features, Options, OutputKind, Target, Toolchain};

mod format;
pub use format::Format;

mod extract;
pub use extract::{Extract, Long, Short};

pub fn find_cargo(toolchain: Toolchain) -> Option<String> {
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
