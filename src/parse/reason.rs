use super::Message;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum Reason {
    CompilerMessage {
        message: Message,
    },
    BuildFinished {
        success: bool,
    },
    #[serde(other)]
    Ignored,
}

impl Reason {
    pub fn as_message(&self) -> Option<&Message> {
        let Self::CompilerMessage { message } = self else {
            return None;
        };
        Some(message)
    }

    pub fn as_locations(&self) -> Option<impl Iterator<Item = (String, String)> + use<'_>> {
        self.as_message().map(Message::as_locations)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Ignored | Self::BuildFinished { .. })
            || matches!(self, Self::CompilerMessage { message } if message.spans.is_empty() && message.children.is_empty())
    }

    #[inline]
    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}
