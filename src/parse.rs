use std::path::PathBuf;

use crate::{args::Sort, options::Filters};

pub fn gather_reasons(output: impl std::io::Read) -> Vec<Reason> {
    serde_json::Deserializer::from_reader(output)
        .into_iter()
        .flatten()
        .filter(Reason::is_not_empty)
        .collect::<_>()
}

pub fn gather_reasons_by(output: impl std::io::Read, filter: &Filters) -> Vec<Reason> {
    let mut reasons = gather_reasons(output);
    reasons.retain(|reason| {
        let Reason::CompilerMessage { message } = &reason else {
            return true;
        };
        let name = message.code.as_ref().map(|c| &*c.code);
        !filter.is_ignored(message.level, name)
    });
    reasons
}

pub fn sort_reasons_by(reasons: &mut [Reason], sort: Sort) {
    match sort {
        Sort::File => sort_by_file(reasons),
        Sort::Lint => sort_by_lint(reasons),
        Sort::Default => {}
    }
}

fn sort_by_file(reasons: &mut [Reason]) {
    reasons.sort_unstable_by_key(|k| {
        k.as_message().map(|c| {
            c.spans
                .iter()
                .map(|s| {
                    let (file, _) = s.as_location();
                    PathBuf::from(file)
                })
                .collect::<Vec<_>>()
        })
    });
}

fn sort_by_lint(reasons: &mut [Reason]) {
    reasons.sort_unstable_by_key(|key| {
        dbg!(key.as_message().map(|msg| Key {
            level: msg.level,
            code: msg.code.clone(),
        }))
    });
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    level: Level,
    code: Option<Code>,
}

mod reason;
pub use reason::Reason;

mod message;
pub use message::Message;

mod level;
pub use level::Level;

mod code;
pub use code::Code;

pub mod span;
pub use span::Span;

mod text;
pub use text::Text;
