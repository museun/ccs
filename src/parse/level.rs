#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
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
