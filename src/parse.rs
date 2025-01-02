use std::{cmp::Ordering, path::PathBuf};

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

pub fn sort_reasons_by(reasons: &mut Vec<Reason>, sort: Sort) {
    match sort {
        Sort::File => sort_by_file(reasons),
        Sort::Lint => sort_by_lint(reasons),
        Sort::Default => {}
    }
}

// #[derive(Debug, Default)]
// struct Trie {
//     trie: BTreeMap<PathBuf, Self>,
// }

// impl Trie {
//     fn is_single_dir(&self) -> bool {
//         self.trie.len() == 0 && !self.trie.iter().next().unwrap().1.trie.is_empty()
//     }

//     fn push(&mut self, path: impl AsRef<Path>) {
//         let mut current = self;
//         for part in path.as_ref().iter() {
//             current = current
//                 .trie
//                 .entry(PathBuf::from(part))
//                 .or_insert_with(Self::default)
//         }
//     }
// }

fn sort_by_file(reasons: &mut Vec<Reason>) {
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

fn sort_by_lint(reasons: &mut Vec<Reason>) {
    reasons.sort_unstable_by(|l, r| match (l.as_message(), r.as_message()) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => {
            // TODO errors are kind of different
            left.code.cmp(&right.code)
        }
    });
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
