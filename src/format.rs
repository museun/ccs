pub trait Format {
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()>;
}

impl<F> Format for Vec<F>
where
    F: Format,
{
    fn format(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.iter().try_for_each(|el| el.format(w))
    }
}
