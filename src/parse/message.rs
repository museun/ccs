use crate::IncludeNotes;

use super::{Code, Level, Span};

// TODO handle notes
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Message {
    pub code: Option<Code>,
    pub message: String,
    pub level: Level,
    pub spans: Vec<Span>,
    // TODO this is never used. the notes are in this
    pub children: Vec<Self>,
}

impl Message {
    pub fn expect_code(&self) -> &str {
        self.code.as_ref().map(|c| &c.code).unwrap()
    }

    pub fn as_locations(&self) -> impl Iterator<Item = (String, String)> + use<'_> {
        self.spans.iter().map(|c| c.as_location())
    }

    pub fn should_ignore(&self, include: IncludeNotes) -> bool {
        matches!(self.level, Level::Note) && matches!(include, IncludeNotes::No)
    }
}
