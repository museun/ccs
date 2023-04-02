#[derive(Copy, Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Warning,
    Error,
    FailureNote,
    Help,
    Note,
    #[serde(other)]
    Unknown,
}
