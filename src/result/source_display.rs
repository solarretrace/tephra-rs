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
        if self.color_enabled {
            writeln!(f, "{}{} {}", 
                self.message_type,
                ":".bright_white().bold(),
                self.message.bright_white().bold())?;
        } else {
            self.message_type
                .write_with_color_enablement(f, self.color_enabled)?;
            writeln!(f, ": {}", self.message)?;
        }
        for source_span in &self.source_spans {
            source_span.write_with_color_enablement(f, self.color_enabled)?;
        }
        for note in &self.notes {
            note.write_with_color_enablement(f, self.color_enabled)?;
        }
        Ok(())
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
    /// Whether colors are enabled during writing.
    color_enabled: bool,
}

impl<'text, 'msg, Cm> SourceSpan<'text, 'msg, Cm> where Cm: ColumnMetrics {
    /// Constructs a new SourceSpan with the given span.
    pub fn new(span: Span<'text>, metrics: Cm) -> Self
        where Cm: ColumnMetrics,
    {
        let gutter_width = std::cmp::max(
            (span.end().page.line as f32).log10().ceil() as usize, 1);

        SourceSpan {
            source_name: None,
            metrics,
            span: span.widen_to_line(metrics),
            highlights: Vec::with_capacity(2),
            notes: Vec::new(),
            allow_omissions: true,
            gutter_width,
            color_enabled: true,
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
        SourceSpan::new(span, metrics)
            .with_highlight(Highlight::new(span, message)
                .with_error_type())
    }

    /// Returns the given SourceDisplay with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.color_enabled = color_enabled;
        self
    }

    /// Returns the given SourceSpan with the given source name.
    pub fn with_source_name<M>(mut self, name: M) -> Self
        where M: Into<Cow<'msg, str>>,
    {
        self.source_name = Some(name.into());
        self
    }

    /// Attaches the given Highlight to the source span.
    pub fn with_highlight(mut self, highlight: Highlight<'text, 'msg>) -> Self {
        self.highlights.push(highlight);
        self
    }

    /// Attaches the given SourceNote to the source span.
    pub fn with_note(mut self, note: SourceNote<'msg>) -> Self {
        self.notes.push(note);
        self
    }

    fn write_with_color_enablement(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        color_enabled: bool)
        -> std::fmt::Result
    {
        let (source_name, sep) = match &self.source_name {
            Some(name) => (name.borrow(), ":"),
            None       => ("", ""),
        };

        if color_enabled {
            writeln!(f, "{:width$}{} {}{}({})",
                "",
                "-->".bright_blue().bold(),
                source_name,
                sep,
                self.span,
                width=self.gutter_width)?;
        } else {
            writeln!(f, "{:width$}--> {}{}({})",
                "",
                source_name,
                sep,
                self.span,
                width=self.gutter_width)?;
        }

        MultiSplitLines::new(
            self.span,
            &self.highlights[..],
            self.gutter_width,
            self.metrics)
            .write_with_color_enablement(f, color_enabled)?;

        for note in &self.notes {
            write!(f, "{:width$} = ", "", width=self.gutter_width)?;
            note.write_with_color_enablement(f, color_enabled)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<'text, 'msg, Cm> Display for SourceSpan<'text, 'msg, Cm> 
    where Cm: ColumnMetrics,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, self.color_enabled)
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

impl<'msg> SourceNote<'msg> {
    fn write_with_color_enablement(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        color_enabled: bool)
        -> std::fmt::Result
    {
        self.note_type.write_with_color_enablement(f, color_enabled)?;
        write!(f, ": {}", self.note)
    }
}

impl<'msg> Display for SourceNote<'msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, true)
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

    fn write_with_color_enablement(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        color_enabled: bool)
        -> std::fmt::Result
    {
        use MessageType::*;
        if color_enabled {
            let color = self.color();
            
            match self {
                Info    => write!(f, "{}", "info"),
                Error   => write!(f, "{}", "error".color(color).bold()),
                Warning => write!(f, "{}", "warning".color(color).bold()),
                Note    => write!(f, "{}", "note".color(color).bold()),
                Help    => write!(f, "{}", "help".color(color).bold()),
            }
        } else {
            match self {
                Info    => write!(f, "info"),
                Error   => write!(f, "error"),
                Warning => write!(f, "warning"),
                Note    => write!(f, "note"),
                Help    => write!(f, "help"),
            }
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, true)
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
        current_line: usize,
        riser_state: &mut RiserState,
        is_active_riser: bool,
        color_enabled: bool)
        -> std::fmt::Result
    {
        // If the span is not over multiple lines, there is no riser portion.
        if *riser_state == RiserState::Unused { return Ok(()); }

        match *riser_state {
            RiserState::Unused  => Ok(()),

            RiserState::Ended   => write!(f, " "),

            RiserState::Waiting => if !is_active_riser 
                && self.span.start().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Started;
                if color_enabled {
                    write!(f, "{}", "/".color(self.message_type.color()))
                } else {
                    write!(f, "/")
                }

            } else if self.has_message_for_line(current_line) {
                *riser_state = RiserState::Started;
                write!(f, " ")

            } else {
                write!(f, " ")
            },

            RiserState::Started => if !is_active_riser 
                && self.span.end().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Ended;
                if color_enabled {
                    write!(f, "{}", "\\".color(self.message_type.color()))
                } else {
                    write!(f, "\\")
                }

            } else if self.has_message_for_line(current_line) {
                if is_active_riser { *riser_state = RiserState::Ended; }
                write!(f, "|")

            } else {
                write!(f, "|")
            },
        }

    }

    /// Writes the message text for the given line number.
    fn write_message_for_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        line: usize,
        write_extra_riser_spacer: bool,
        color_enabled: bool)
        -> std::fmt::Result
    {
        if self.span.start().page.line == line
            && self.span.end().page.line == line
        {
            if write_extra_riser_spacer { write!(f, " ")?; }
            for _ in 0..self.span.start().page.column {
                write!(f, " ")?;
            }
            if self.span.is_empty() {
                if color_enabled {
                    write!(f, "{}", "\\".color(self.message_type.color()))?;
                } else {
                    write!(f, "\\")?;
                }
            } else {
                let mut underline_count = std::cmp::max(
                    self.span.end().page.column
                        .checked_sub(self.span.start().page.column)
                        .unwrap_or(0),
                    1);
                for _ in 0..underline_count {
                    if color_enabled {
                        write!(f, "{}", self.message_type
                            .underline()
                            .color(self.message_type.color()))?;
                    } else {
                        write!(f, "{}", self.message_type.underline())?;
                    }
                }
            }
            match (&self.start_message, &self.end_message) {
                (Some(msg), None)      | 
                (None,      Some(msg)) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?
                } else {
                    writeln!(f, " {}", msg)?
                },
                (Some(fst), Some(snd)) => unimplemented!(),
                (None,      None)      => writeln!(f, "")?,
            }

        }  else if self.span.start().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(f, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(f, "_")?;
                }
            }
            if self.span.start().page.column > 0 {
                for _ in 0..(self.span.start().page.column - 1) {
                    if color_enabled {
                        write!(f, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(f, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(f, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(f, "^")?;
            }
            match &self.start_message {
                Some(msg) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?;
                } else {
                    write!(f, " {}", msg)?;
                },
                None      => writeln!(f, "")?,
            }
            
        } else if self.span.end().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(f, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(f, "_")?;
                }
            }
            if self.span.end().page.column > 0 {
                for _ in 0..(self.span.end().page.column - 1) {
                    if color_enabled {
                        write!(f, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(f, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(f, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(f, "^")?;
            }
            match &self.end_message {
                Some(msg) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?;
                } else {
                    writeln!(f, " {}", msg)?;
                },
                
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

    /// Consumes the MultiSplitLines and writes all of the contained data.
    fn write_with_color_enablement(
        mut self,
        f: &mut std::fmt::Formatter<'_>,
        color_enabled: bool)
        -> std::fmt::Result
    {
        // Write empty line to uncramp the display.
        write_gutter(f, "", self.gutter_width, color_enabled)?;
        writeln!(f)?;

        let mut riser_states = Vec::with_capacity(self.highlights.len());
        for hl in self.highlights {
            riser_states
                .push(if hl.is_multiline() { 
                    RiserState::Waiting
                } else {
                    RiserState::Unused
                });
        }

        for span in self.source_lines {
            let current_line = span.start().page.line;

            // Write gutter for source line.
            write_gutter(f, current_line, self.gutter_width, color_enabled)?;

            // Write risers for source line.
            let mut any_multiline = false;
            for (idx, hl) in self.highlights.iter().enumerate() {
                hl.write_riser_for_line(
                    f,
                    current_line,
                    &mut riser_states[idx],
                    false,
                    color_enabled)?;
                if hl.is_multiline() { any_multiline = true; }
            }
            if any_multiline { write!(f, " "); }

            // Write source text.
            writeln!(f, "{}", span.text())?;

            for (message_idx, message_hl) in self.highlights.iter().enumerate()
            {
                if !message_hl.has_message_for_line(current_line) { continue; }

                // Write message gutter.
                write_gutter(f, "", self.gutter_width, color_enabled)?;
                
                for (idx, hl) in self.highlights.iter().enumerate() {
                    // Write message risers.
                    hl.write_riser_for_line(
                        f,
                        current_line,
                        &mut riser_states[idx],
                        message_idx == idx,
                        color_enabled)?;
                }
                
                // Write message.
                message_hl.write_message_for_line(
                    f,
                    current_line,
                    any_multiline,
                    color_enabled)?;
            }
        }
        Ok(())
    }
}

fn write_gutter<V>(
    f: &mut std::fmt::Formatter<'_>,
    value: V,
    width: usize,
    color_enabled: bool)
    -> std::fmt::Result
    where V: Display
{
    if color_enabled {
        write!(f, "{:>width$} {} ",
            format!("{}", value).bright_blue().bold(),
            "|".bright_blue().bold(),
            width=width)
    } else {
        write!(f, "{:>width$} | ", value, width=width)
    }
}

fn write_gutter_omit<V>(
    f: &mut std::fmt::Formatter<'_>,
    value: V,
    width: usize,
    color_enabled: bool)
    -> std::fmt::Result
    where V: Display
{
    if color_enabled {
        write!(f, "{:>width$}{}",
            "",
            "...".bright_blue().bold(),
            width=width)
    } else {
        write!(f, "{:>width$}...", "", width=width)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiserState {
    Unused,
    Waiting,
    Started,
    Ended,
}
