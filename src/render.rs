#[derive(Default, Copy, Clone)]
pub enum Render {
    #[default]
    Short,
    Full,
}

#[derive(Default, Copy, Clone)]
pub enum IncludeNotes {
    Yes,
    #[default]
    No,
}
