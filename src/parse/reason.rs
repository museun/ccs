use std::borrow::Cow;

use crate::{RenderOptions, Theme};

use super::Message;

#[derive(Debug, serde::Deserialize)]
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
    pub fn render(
        &self,
        render_options: &RenderOptions,
        theme: &Theme,
        continuation: &Option<Cow<'static, str>>,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        match self {
            Self::CompilerMessage { message } => {
                message.render(render_options, theme, continuation, out)
            }
            Self::BuildFinished { success: true } => {
                // TODO perhaps report this with a flag
                Ok(())
            }
            Self::BuildFinished { success: false } => {
                // TODO perhaps report this with a flag
                Ok(())
            }
            _ => Ok(()),
        }
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
