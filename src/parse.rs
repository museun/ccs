pub fn gather_reasons(output: impl std::io::Read) -> Vec<Reason> {
    serde_json::Deserializer::from_reader(output)
        .into_iter()
        .flatten()
        .filter(Reason::is_not_empty)
        .collect::<_>()
}

mod reason;
pub use reason::Reason;

mod message;
pub use message::Message;

mod level;
pub use level::Level;

mod code;
pub use code::Code;

mod span;
pub use span::Span;

mod text;
pub use text::Text;
