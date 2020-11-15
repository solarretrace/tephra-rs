////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::span::Span;
use crate::span::SplitLines;
use crate::position::ColumnMetrics;

// External library imports.
use colored::Color;
use colored::Colorize as _;

// Standard library imports.
use std::borrow::Cow;
use std::borrow::Borrow as _;
use std::fmt::Display;



fn with_color_override<F>(color_enable: bool, f: F) -> std::fmt::Result
    where F: FnOnce() -> std::fmt::Result
{
    colored::control::set_override(color_enable);
    let res = (f)();
    colored::control::unset_override();
    res
}

////////////////////////////////////////////////////////////////////////////////
// SourceDisplay
////////////////////////////////////////////////////////////////////////////////
/// A structure for displaying source text with spans, notes, and highlights.
#[derive(Debug)]
pub struct SourceDisplay<'text, 'msg, Cm> {
    /// The top-level description for all of the source spans.
    message: Cow<'msg, str>,
    /// The overall message type for all of the source spans.
    message_type: MessageType,
    /// The source spans to display.
    source_spans: Vec<SourceSpan<'text, 'msg, Cm>>,
    /// Notes to append after the displayed spans.
    notes: Vec<SourceNote<'msg>>,
    /// Whether colors are enabled during writing.
    color_enabled: bool,
}

impl<'text, 'msg, Cm> SourceDisplay<'text, 'msg, Cm> {
    /// Constructs a new info-type SourceDisplay with the given description.
    pub fn new<M>(message: M) -> Self 
        where M: Into<Cow<'msg, str>>,
    {
        SourceDisplay {
            message: message.into(),
            message_type: MessageType::Info,
            source_spans: Vec::with_capacity(1),
            notes: Vec::new(),
            color_enabled: true,
        }
    }

    /// Returns the given SourceDisplay with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.color_enabled = color_enabled;
        self
    }


    /// Returns the given SourceDisplay with the error MessageType.
    pub fn with_error_type(mut self) -> Self {
        self.message_type = MessageType::Error;
        self
    }

    /// Returns the given SourceDisplay with the warning MessageType.
    pub fn with_warning_type(mut self) -> Self {
        self.message_type = MessageType::Warning;
        self
    }
    
    /// Returns the given SourceDisplay with the note MessageType.
    pub fn with_note_type(mut self) -> Self {
        self.message_type = MessageType::Note;
        self
    }

    /// Returns the given SourceDisplay with the hel MessageType.
    pub fn with_help_type(mut self) -> Self {
        self.message_type = MessageType::Help;
        self
    }

    /// Returns the given SourceDisplay with the given MessageType.
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Returns the given SourceDisplay with the given SourceSpan attachment.
    pub fn with_source_span<S>(mut self, source_span: S)
        -> Self
        where S: Into<SourceSpan<'text, 'msg, Cm>>
    {
        self.source_spans.push(source_span.into());
        self
    }

    /// Returns the given SourceDisplay with the given note attachment.
    pub fn with_note<N>(mut self, note: N)
        -> Self
        where N: Into<SourceNote<'msg>>
    {
        self.notes.push(note.into());
        self
    }
}

impl<'text, 'msg, Cm> Display for SourceDisplay<'text, 'msg, Cm> 
    where Cm: ColumnMetrics,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        with_color_override(self.color_enabled, || {
            writeln!(f, "{}{} {}", 
                self.message_type,
                ":".bright_white().bold(),
                self.message.bright_white().bold())?;
            for source_span in &self.source_spans {
                writeln!(f, "{}", source_span)?;
            }
            for note in &self.notes {
                writeln!(f, "{}", note)?;
            }
            Ok(())
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// SourceSpan
////////////////////////////////////////////////////////////////////////////////
/// A single span of source text with notes and highlights.
#[derive(Debug)]
pub struct SourceSpan<'text, 'msg, Cm> {
    /// The name of the file or data that is being displayed.
    source_name: Option<Cow<'msg, str>>,
    /// The column metrics for the source,
    metrics: Cm,
    /// The full text span of the displayed source.
    span: Span<'text>,
    /// The subsets of the displayed text to highlight.
    highlights: Vec<Highlight<'text, 'msg>>,
    /// Notes to append to the source display.
    notes: Vec<SourceNote<'msg>>,
    /// Whether to allow line omissions within the source display.
    allow_omissions: bool,
    /// The width of the line number gutter.
    gutter_width: usize,
}

impl<'text, 'msg, Cm> SourceSpan<'text, 'msg, Cm> {
    /// Constructs a new SourceSpan with the given span.
    pub fn new(span: Span<'text>, metrics: Cm) -> Self
        where Cm: ColumnMetrics,
    {
        SourceSpan {
            source_name: None,
            metrics,
            span: span.widen_to_line(metrics),
            highlights: Vec::with_capacity(2),
            notes: Vec::new(),
            allow_omissions: true,
            gutter_width: 0,
        }
    }

    /// Constructs a new SourceSpan with the given span and highlight message.
    pub fn new_error_highlight<M>(
        span: Span<'text>,
        message: M,
        metrics: Cm)
        -> Self
        where
            M: Into<Cow<'msg, str>>,
            Cm: ColumnMetrics,
    {
        let highlight = Highlight::new(span, message)
            .with_error_type();
        let gutter_width = std::cmp::max(
            (span.end().page.line as f32).log10().ceil() as usize, 1);

        SourceSpan {
            source_name: None,
            metrics,
            span: span.widen_to_line(metrics),
            highlights: vec![highlight],
            notes: Vec::new(),
            allow_omissions: true,
            gutter_width,
        }
    }

    /// Returns the given SourceSpan with the given source name.
    pub fn with_source_name<M>(mut self, name: M) -> Self
        where M: Into<Cow<'msg, str>>,
    {
        self.source_name = Some(name.into());
        self
    }
}

impl<'text, 'msg, Cm> Display for SourceSpan<'text, 'msg, Cm> 
    where Cm: ColumnMetrics,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (source_name, sep) = match &self.source_name {
            Some(name) => (name.borrow(), ":"),
            None       => ("", ""),
        };

        if self.span.start() == self.span.end() {
            writeln!(f, "{:width$}{} {}{}({}, byte {})",
                "",
                "-->".bright_blue().bold(),
                source_name,
                sep,
                self.span.start().page,
                self.span.start().byte,
                width=self.gutter_width)?;
        } else {
            writeln!(f, "{:width$}{} {}{}({}-{}, bytes {}-{})",
                "",
                "-->".bright_blue().bold(),
                source_name,
                sep,
                self.span.start().page,
                self.span.end().page,
                self.span.start().byte,
                self.span.end().byte,
                width=self.gutter_width)?;
        }

        MultiSplitLines::new(
            self.span,
            &self.highlights[..],
            self.gutter_width,
            self.metrics)
            .write_all(f)?;

        for note in &self.notes {
            writeln!(f, "{:width$} = {}",
                "",
                note,
                width=self.gutter_width)?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// SourceNote
////////////////////////////////////////////////////////////////////////////////
/// A note which can be attached to a `SourceSpan` or `SourceDisplay`.
#[derive(Debug)]
pub struct SourceNote<'msg> {
    /// The message type for the note.
    note_type: MessageType,
    /// The note to display.
    note: Cow<'msg, str>,
}

impl<'msg> Display for SourceNote<'msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.note_type, self.note)
    }
}

////////////////////////////////////////////////////////////////////////////////
// MessageType
////////////////////////////////////////////////////////////////////////////////
/// A `SourceDisplay`, `SourceNote`, or `Highlight` message type. Used to
/// determine the color and format of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// An informational message.
    Info,
    /// An error message.
    Error,
    /// A warning message.
    Warning,
    /// A message providing additional info.
    Note,
    /// A message to help in correcting an error or warning.
    Help,
}

impl MessageType {
    /// Returns the color associated with the message type.
    pub fn color(&self) -> Color {
        use MessageType::*;
        match self {
            Info    => Color::BrightWhite,
            Error   => Color::BrightRed,
            Warning => Color::BrightYellow,
            Note    => Color::BrightBlue,
            Help    => Color::BrightGreen,
        }
    }

    /// Returns the underline associated with the message type.
    pub fn underline(&self) -> &'static str {
        use MessageType::*;
        match self {
            Info    => "-",
            Error   => "^",
            Warning => "^",
            Note    => "-",
            Help    => "~",
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.color();
        use MessageType::*;
        match self {
            Info    => write!(f, "{}", "info"),
            Error   => write!(f, "{}", "error".color(color).bold()),
            Warning => write!(f, "{}", "warning".color(color).bold()),
            Note    => write!(f, "{}", "note".color(color).bold()),
            Help    => write!(f, "{}", "help".color(color).bold()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Highlight
////////////////////////////////////////////////////////////////////////////////
/// A highlighted subsection of a `SourceSpan`.
#[derive(Debug)]
pub struct Highlight<'text, 'msg> {
    /// The span to highlight.
    span: Span<'text>,
    /// The message to display at the start of the span.
    start_message: Option<Cow<'msg, str>>,
    /// The message to display at the end of the span.
    end_message: Option<Cow<'msg, str>>,
    /// The message type.
    message_type: MessageType,
    /// Whether to allow line omissions within the highlighted span.
    allow_omissions: bool,
}


impl<'text, 'msg> Highlight<'text, 'msg> {
    /// Constructs a new Highlight with the given span and message.
    pub fn new<M>(span: Span<'text>, message: M) -> Self
        where M: Into<Cow<'msg, str>>,
    {
        Highlight {
            span,
            start_message: None,
            end_message: Some(message.into()),
            message_type: MessageType::Info,
            allow_omissions: true,
        }
    }

    /// Returns the given SourceDisplay with the info MessageType.
    pub fn with_info_type(mut self) -> Self {
        self.message_type = MessageType::Info;
        self
    }

    /// Returns the given SourceDisplay with the error MessageType.
    pub fn with_error_type(mut self) -> Self {
        self.message_type = MessageType::Error;
        self
    }

    /// Returns the given SourceDisplay with the warning MessageType.
    pub fn with_warning_type(mut self) -> Self {
        self.message_type = MessageType::Warning;
        self
    }
    
    /// Returns the given SourceDisplay with the note MessageType.
    pub fn with_note_type(mut self) -> Self {
        self.message_type = MessageType::Note;
        self
    }

    /// Returns the given SourceDisplay with the hel MessageType.
    pub fn with_help_type(mut self) -> Self {
        self.message_type = MessageType::Help;
        self
    }

    /// Returns the given SourceDisplay with the given MessageType.
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Returns true if the highlight extends across multiple lines.
    pub fn is_multiline(&self) -> bool {
        self.span.start().page.line != self.span.end().page.line
    }

    /// Returns true if the highlight has a message for the given line.
    pub fn has_message_for_line(&self, line: usize) -> bool {
        (self.span.start().page.line == line
            && (self.start_message.is_some() 
                || self.span.start().page.column != 0))
        ||
        (self.span.end().page.line == line
            && (self.end_message.is_some() 
                || self.span.end().page.column != 0))
    }

    /// Writes the riser symbol for the given line number.
    fn write_riser_for_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        line: usize,
        is_message_line: bool)
        -> std::fmt::Result
    {
        if !self.is_multiline() { return Ok(()) }
        if self.span.start().page.line == line {
            if self.span.start().page.column == 0 && !is_message_line {
                write!(f, "{}", "/".color(self.message_type.color()))
            } else if is_message_line {
                write!(f, "{}", "|".color(self.message_type.color()))
            } else {
                write!(f, " ")
            }

        } else if self.span.start().page.line > line
            && self.span.end().page.line < line
        {
            write!(f, "{}", "|".color(self.message_type.color()))

        } else if self.span.end().page.line == line {
            if self.span.end().page.column == 0 && !is_message_line {
                write!(f, "{}", "\\".color(self.message_type.color()))
            } else  {
                write!(f, "{}", "|".color(self.message_type.color()))
            }
        } else {
            Ok(())
        }
    }

    /// Writes the message text for the given line number.
    fn write_message_for_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        line: usize)
        -> std::fmt::Result
    {
        if self.span.start().page.line == line
            && self.span.end().page.line == line
        {
            for _ in 0..self.span.start().page.column {
                write!(f, " ")?;
            }
            let underline_count = std::cmp::max(
                self.span.end().page.column
                    .checked_sub(self.span.start().page.column)
                    .unwrap_or(0),
                1);
            for _ in 0..underline_count {
                write!(f, "{}", self.message_type
                    .underline()
                    .color(self.message_type.color()))?;
            }
            match (&self.start_message, &self.end_message) {
                (Some(msg), None)      | 
                (None,      Some(msg)) => writeln!(f, " {}", msg
                    .color(self.message_type.color()))?,
                (Some(fst), Some(snd)) => unimplemented!(),
                (None,      None)      => writeln!(f, "")?,
            }

        }  else if self.span.start().page.line == line {
            for _ in 0..self.span.start().page.column {
                write!(f, "{}", "_".color(self.message_type.color()))?;
            }
            write!(f, "{}", "^".color(self.message_type.color()))?;
            match &self.start_message {
                Some(msg) => writeln!(f, " {}", msg
                    .color(self.message_type.color()))?,
                None      => writeln!(f, "")?,
            }
            
        } else if self.span.end().page.line == line {
            for _ in 0..(self.span.end().page.column - 1) {
                write!(f, "{}", "_".color(self.message_type.color()))?;
            }
            write!(f, "{}", "^".color(self.message_type.color()))?;
            match &self.end_message {
                Some(msg) => writeln!(f, " {}", msg
                    .color(self.message_type.color()))?,
                None      => writeln!(f, "")?,
            }
        }
        Ok(())
    }
}



////////////////////////////////////////////////////////////////////////////////
// MultiSplitLines
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the line-based data relevant to a particular SourceSpan.
#[derive(Debug)]
struct MultiSplitLines<'text, 'msg, 'hl, Cm> {
    /// The SplitLines iterator for the `SourceSpan`.
    source_lines: SplitLines<'text, Cm>,
    /// The highlights contained within the SourceSpan.
    highlights: &'hl [Highlight<'text, 'msg>],
    /// The width of the line number gutter.
    gutter_width: usize,
    /// The width of the highlight riser gutter.
    riser_width: usize,
}

impl<'text, 'msg, 'hl, Cm> MultiSplitLines<'text, 'msg, 'hl, Cm> 
    where
        'text: 'msg,
        Cm: ColumnMetrics,
{
    /// Constructs a new MultiSplitLines from the given source span and
    /// highlights.
    fn new(
        source_span: Span<'text>,
        highlights: &'hl [Highlight<'text, 'msg>],
        gutter_width: usize,
        metrics: Cm)
        -> Self
    {
        let riser_width = highlights
            .iter()
            .filter(|h| h.is_multiline())
            .count();

        let source_lines = source_span.split_lines(metrics);

        MultiSplitLines {
            source_lines,
            highlights,
            gutter_width,
            riser_width,
        }
    }

    /// Consumes the
    fn write_all(mut self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let hl_count = self.highlights.len();

        // Write empty line to uncramp the display.
        write_gutter(f, "", self.gutter_width)?;
        writeln!(f, "")?;

        for span in self.source_lines {
            let current_line = span.start().page.line;

            // Write source gutter.
            write_gutter(f, current_line, self.gutter_width)?;

            for hl in self.highlights {
                // Write source risers.
                hl.write_riser_for_line(f, current_line, false)?;
            }

            // Write source.
            write_source_line(f, span)?;

            if self.highlights
                .iter()
                .any(|hl| hl.has_message_for_line(current_line))
            {
                // Write message gutter.
                write_gutter(f, "", self.gutter_width)?;
                for hl in self.highlights {
                    // Write message risers.
                    hl.write_riser_for_line(f, current_line, true)?;
                    // Write message.
                    hl.write_message_for_line(f, current_line)?;
                }
            }
        }
        Ok(())
    }
}

fn write_gutter<V>(
    f: &mut std::fmt::Formatter<'_>,
    value: V,
    width: usize)
    -> std::fmt::Result
    where V: Display
{
    write!(f, "{:>width$} {} ",
        format!("{}", value).bright_blue().bold(),
        "|".bright_blue().bold(),
        width=width)
}

fn write_gutter_omit<V>(
    f: &mut std::fmt::Formatter<'_>,
    value: V,
    width: usize)
    -> std::fmt::Result
    where V: Display
{
    write!(f, "{:>width$}{}",
        "",
        "...".bright_blue().bold(),
        width=width)
}


fn write_source_line<'text>(
    f: &mut std::fmt::Formatter<'_>,
    span: Span<'text>)
    -> std::fmt::Result {
    writeln!(f, "{}", span.text())?;
    Ok(())
}
