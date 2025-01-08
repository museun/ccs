use std::collections::BTreeMap;

use owo_colors::OwoColorize as _;

use crate::{
    span::Span, Code, IncludeNotes, Level, Message, Reason, RenderOptions, RenderStyle, Theme,
};

struct RenderKind;
#[allow(non_upper_case_globals)]
impl RenderKind {
    const Default: u8 = 0;
    const ByFile: u8 = 1 << 0;
    const ByLint: u8 = 1 << 1;
    const Unknown: u8 = u8::MAX;
}

pub(super) struct Renderer<'a, const KIND: u8 = { RenderKind::Unknown }> {
    options: &'a RenderOptions,
    theme: &'a Theme,
    out: &'a mut dyn std::io::Write,
}

impl<'a> Renderer<'a, { RenderKind::Unknown }> {
    pub(super) fn new(
        options: &'a RenderOptions,
        theme: &'a Theme,
        out: &'a mut dyn std::io::Write,
    ) -> Self {
        Renderer {
            options,
            theme,
            out,
        }
    }

    pub(super) const fn group_by_default(self) -> Renderer<'a, { RenderKind::Default }> {
        Renderer {
            options: self.options,
            theme: self.theme,
            out: self.out,
        }
    }

    pub(super) const fn group_by_file(self) -> Renderer<'a, { RenderKind::ByFile }> {
        Renderer {
            options: self.options,
            theme: self.theme,
            out: self.out,
        }
    }

    pub(super) const fn group_by_lint(self) -> Renderer<'a, { RenderKind::ByLint }> {
        Renderer {
            options: self.options,
            theme: self.theme,
            out: self.out,
        }
    }
}

impl Renderer<'_, { RenderKind::Default }> {
    pub(super) fn render(mut self, reasons: &[Reason]) -> std::io::Result<()> {
        for (i, reason) in reasons.iter().enumerate() {
            self.reason(i, reason)?;
        }
        Ok(())
    }

    fn reason(&mut self, index: usize, reason: &Reason) -> std::io::Result<()> {
        maybe_new_line(self.out, self.options, index)?;

        match reason {
            Reason::CompilerMessage { message } => self.message(message),
            // TODO perhaps report this with a flag
            Reason::BuildFinished { success: true } => Ok(()),
            // TODO perhaps report this with a flag
            Reason::BuildFinished { success: false } => Ok(()),
            Reason::Ignored => Ok(()),
        }
    }

    fn message(&mut self, msg: &Message) -> std::io::Result<()> {
        if !msg.should_ignore(self.options.include_notes) {
            self.header(msg)?;
        }

        for span in &msg.spans {
            if msg.should_ignore(self.options.include_notes) {
                continue;
            }

            self.span(span)?;

            if let Some(Code { ref code }) = msg
                .code
                .as_ref()
                .filter(|_| matches!(msg.level, Level::Warning))
            {
                writeln!(
                    self.out,
                    "({code})",
                    code = code.color(self.theme.lint_name)
                )?
            } else {
                writeln!(self.out)?
            }
        }

        Ok(())
    }

    fn header(&mut self, msg: &Message) -> std::io::Result<()> {
        let color = match msg.level {
            Level::Warning => self.theme.warning,
            Level::Error => self.theme.error,
            Level::Note => self.theme.note,
            _ => self.theme.unknown,
        };

        match msg.level {
            Level::Error if msg.code.is_some() => {
                write!(self.out, "{error} ", error = msg.expect_code().color(color))?;
            }
            Level::Error => {
                write!(self.out, "{error} ", error = "error".color(color))?;
            }
            Level::Warning => {
                write!(self.out, "{warning} ", warning = "warning".color(color))?;
            }
            Level::Note if matches!(self.options.include_notes, IncludeNotes::Yes) => {
                write!(self.out, "{note} ", note = "note".color(color))?;
            }

            // TODO this (this should also be part of --include)
            // Level::Help => {}
            _ => {}
        }

        writeln!(
            self.out,
            "{message}",
            message = msg.message.color(self.theme.message)
        )
    }

    fn span(&mut self, span: &Span) -> std::io::Result<()> {
        if matches!(self.options.render, RenderStyle::Full) {
            for (head, mid, tail) in span.partition_explain() {
                writeln!(
                    self.out,
                    "  {head}{mid}{tail}",
                    head = head.color(self.theme.code),
                    mid = mid.color(self.theme.highlight),
                    tail = tail.color(self.theme.code)
                )?
            }
        }

        let (location, pos) = span.as_location();

        match &self.options.continuation {
            Some(continuation) => {
                // TODO don't prepend the space before the continuation -- the
                // user can do it in their configuration.
                // such as: "@" could be set as " @"
                write!(
                    self.out,
                    " {cont} {location}{pos} ",
                    cont = continuation.color(self.theme.continuation),
                    location = location.color(self.theme.location),
                    pos = pos.color(self.theme.position)
                )
            }
            None => write!(
                self.out,
                " {location}{pos} ",
                location = location.color(self.theme.location),
                pos = pos.color(self.theme.position)
            ),
        }
    }
}

impl Renderer<'_, { RenderKind::ByFile }> {
    pub(super) fn render(mut self, reasons: &[Reason]) -> std::io::Result<()> {
        let mut map = <BTreeMap<String, Vec<&Message>>>::default();
        for msg in reasons.iter().flat_map(|c| c.as_message()) {
            for (location, _pos) in msg.as_locations() {
                map.entry(location).or_default().push(msg);
            }
        }

        for (i, (file, grouped)) in map.into_iter().enumerate() {
            maybe_new_line(self.out, self.options, i)?;
            writeln!(self.out, "in file: {file}", file = file.bold().italic())?;

            for (i, msg) in grouped.into_iter().enumerate() {
                if i > 0
                    && self.options.new_line
                    && matches!(self.options.render, RenderStyle::Full)
                {
                    writeln!(self.out)?;
                }
                self.header(msg.level, &msg.code, &msg.message)?;
                self.message(msg)?;
            }
        }

        Ok(())
    }

    fn header(&mut self, level: Level, code: &Option<Code>, msg: &str) -> std::io::Result<()> {
        let color = match level {
            Level::Warning => self.theme.warning,
            Level::Error => self.theme.error,
            Level::Note => self.theme.note,
            _ => self.theme.unknown,
        };

        match level {
            Level::Error if code.is_some() => {
                write!(
                    self.out,
                    "  {error} ",
                    error = code.as_ref().map(|s| &s.code).unwrap().color(color)
                )
            }
            Level::Error => {
                write!(self.out, "  {error} ", error = "error".color(color))
            }
            Level::Warning => {
                write!(self.out, "  {warning} ", warning = "warning".color(color))
            }
            Level::Note if matches!(self.options.include_notes, IncludeNotes::Yes) => {
                write!(self.out, "  {note} ", note = "note".color(color))
            }
            _ => return Ok(()),
        }?;

        writeln!(
            self.out,
            "{message}",
            message = msg.color(self.theme.message)
        )?;

        Ok(())
    }

    fn message(&mut self, message: &Message) -> std::io::Result<()> {
        for (span, (location, pos)) in message.spans.iter().zip(message.as_locations()) {
            if matches!(self.options.render, RenderStyle::Full) {
                for (head, mid, tail) in span.partition_explain() {
                    writeln!(
                        self.out,
                        "    {head}{mid}{tail}",
                        head = head.color(self.theme.code),
                        mid = mid.color(self.theme.highlight),
                        tail = tail.color(self.theme.code)
                    )?
                }
            }
            match &self.options.continuation {
                Some(continuation) => {
                    // TODO don't prepend the space before the continuation -- the
                    // user can do it in their configuration.
                    // such as: "@" could be set as " @"
                    write!(
                        self.out,
                        "  {cont} {location}{pos} ",
                        cont = continuation.color(self.theme.continuation),
                        location = location.color(self.theme.location),
                        pos = pos.color(self.theme.position),
                    )
                }
                None => write!(
                    self.out,
                    "  {location}{pos} ",
                    location = location.color(self.theme.location),
                    pos = pos.color(self.theme.position),
                ),
            }?;
        }

        if let Some(Code { ref code }) = message
            .code
            .as_ref()
            .filter(|_| matches!(message.level, Level::Warning))
        {
            writeln!(
                self.out,
                "({code})",
                code = code.color(self.theme.lint_name)
            )
        } else {
            writeln!(self.out)
        }
    }
}

impl Renderer<'_, { RenderKind::ByLint }> {
    pub(super) fn render(mut self, reasons: &[Reason]) -> std::io::Result<()> {
        let mut map = <BTreeMap<&str, Vec<&Message>>>::default();

        for msg in reasons.iter().flat_map(|c| c.as_message()) {
            map.entry(&msg.message).or_default().push(msg);
        }

        for (i, (msg, grouped)) in map.into_iter().enumerate() {
            maybe_new_line(self.out, self.options, i)?;
            let Some(head) = grouped.first() else {
                continue;
            };
            self.header(head.level, &head.code, msg)?;
            for (i, msg) in grouped.into_iter().enumerate() {
                if i > 0
                    && self.options.new_line
                    && matches!(self.options.render, RenderStyle::Full)
                {
                    writeln!(self.out)?;
                }
                self.message(msg)?;
            }
        }
        Ok(())
    }

    fn header(&mut self, level: Level, code: &Option<Code>, msg: &str) -> std::io::Result<()> {
        let color = match level {
            Level::Warning => self.theme.warning,
            Level::Error => self.theme.error,
            Level::Note => self.theme.note,
            _ => self.theme.unknown,
        };

        match level {
            Level::Error if code.is_some() => {
                write!(
                    self.out,
                    "{error} ",
                    error = code.as_ref().map(|s| &s.code).unwrap().color(color)
                )
            }
            Level::Error => {
                write!(self.out, "{error} ", error = "error".color(color))
            }
            Level::Warning => {
                write!(self.out, "{warning} ", warning = "warning".color(color))
            }
            Level::Note if matches!(self.options.include_notes, IncludeNotes::Yes) => {
                write!(self.out, "{note} ", note = "note".color(color))
            }
            _ => return Ok(()),
        }?;

        writeln!(
            self.out,
            "{message}",
            message = msg.color(self.theme.message)
        )?;

        if matches!(level, Level::Warning) {
            if let Some(Code { ref code }) = code {
                writeln!(self.out, "{code}", code = code.color(self.theme.lint_name))?;
            }
        }

        Ok(())
    }

    fn message(&mut self, message: &Message) -> std::io::Result<()> {
        for (span, (location, pos)) in message.spans.iter().zip(message.as_locations()) {
            if matches!(self.options.render, RenderStyle::Full) {
                for (head, mid, tail) in span.partition_explain() {
                    writeln!(
                        self.out,
                        "  {head}{mid}{tail}",
                        head = head.color(self.theme.code),
                        mid = mid.color(self.theme.highlight),
                        tail = tail.color(self.theme.code)
                    )?
                }
            }
            match &self.options.continuation {
                Some(continuation) => {
                    // TODO don't prepend the space before the continuation -- the
                    // user can do it in their configuration.
                    // such as: "@" could be set as " @"
                    writeln!(
                        self.out,
                        "{cont} {location}{pos} ",
                        cont = continuation.color(self.theme.continuation),
                        location = location.color(self.theme.location),
                        pos = pos.color(self.theme.position),
                    )
                }
                None => writeln!(
                    self.out,
                    "{location}{pos} ",
                    location = location.color(self.theme.location),
                    pos = pos.color(self.theme.position),
                ),
            }?;
        }
        Ok(())
    }
}

fn maybe_new_line(
    out: &mut dyn std::io::Write,
    options: &RenderOptions,
    index: usize,
) -> std::io::Result<()> {
    if index > 0 {
        if let Some(delim) = options.delimiter.as_ref().filter(|c| !c.is_empty()) {
            writeln!(out, "{delim}")?;
        }
        if options.new_line {
            writeln!(out)?;
        }
    }
    Ok(())
}
