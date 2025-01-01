use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{args::Tool, Theme};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Lints {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub warn: Vec<String>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub nightly: bool,
    pub explain: bool,
    pub include_notes: bool,
    pub delimiter: String,
    pub new_line: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tool: Tool,
    pub lints: Lints,
    pub options: Options,
    pub theme: Theme,
    pub continuation: Option<Cow<'static, str>>,
}

impl Config {
    pub const CONTINUATION: Cow<'static, str> = Cow::Borrowed("⮡");
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tool: Default::default(),
            lints: Default::default(),
            options: Default::default(),
            theme: Default::default(),
            continuation: Some(Self::CONTINUATION),
        }
    }
}

impl Config {
    const QUALIFIER: &'static str = "com.github";
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = env!("CARGO_PKG_NAME");
    const CONFIG_FILE_NAME: &'static str = "ccs.toml";

    // TODO: will this ever actually be used?
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let s = toml::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Option<anyhow::Result<Self>> {
        std::fs::read_to_string(path).ok().and_then(|data| {
            toml::from_str(&data)
                .map_err(Into::into)
                .map(Some)
                .transpose()
        })
    }

    pub fn get_config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from(
            Self::QUALIFIER, //
            Self::ORGANIZATION,
            Self::APPLICATION,
        )
        .map(|s| s.config_dir().join(Self::CONFIG_FILE_NAME))
    }
}
