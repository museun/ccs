use std::path::PathBuf;

#[derive(gumdrop::Options, Debug)]
pub struct Args {
    #[options(help_flag, help = "prints the help message")]
    pub help: bool,

    #[options(
        help = "use `clippy::nursery` and `clippy::nursery` (and nightly clippy)",
        default = "false"
    )]
    pub annoying: bool,

    #[options(help = "use nightly`", default = "false")]
    pub nightly: bool,

    #[options(help = "use line breaks", default = "false")]
    pub line_breaks: bool,

    #[options(
        help = "path to a specific Cargo.toml manifest. this defaults to the `cwd`",
        meta = "path"
    )]
    pub path: Option<PathBuf>,

    #[options(help = "check only test targets")]
    pub tests: bool,

    #[options(help = "use the `explain` format")]
    pub explain: bool,

    #[options(
        short = "w",
        long = "warn",
        help = "additional warning lints to use",
        meta = "string"
    )]
    pub additional: Vec<String>,
}
