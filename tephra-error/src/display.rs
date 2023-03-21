////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Internal library imports.
use crate::MessageType;
use crate::Highlight;


// External library imports.
use colored::Color;
use colored::Colorize as _;
use tephra_span::ColumnMetrics;
use tephra_span::Span;
use tephra_span::SplitLines;
use tephra_span::SourceText;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;

// Standard library imports.
use std::borrow::Cow;
use std::fmt::Write;
use std::borrow::Borrow as _;
use std::fmt::Display;


////////////////////////////////////////////////////////////////////////////////
// CodeDisplay
////////////////////////////////////////////////////////////////////////////////
/// A structure for displaying source text with spans, notes, and highlights.
#[derive(Debug, Clone)]
pub struct CodeDisplay {
    /// The top-level description for all of the spans.
    pub(in crate) message: String,
    /// The overall message type for all of the spans.
    pub(in crate) message_type: MessageType,
    /// An error number or warning code to print.
    pub(in crate) code_id: Option<&'static str>,
    /// The spans to display.
    pub(in crate) span_displays: Vec<SpanDisplay>,
    /// Notes to append after the displayed spans.
    pub(in crate) notes: Vec<Note>,
    /// Whether colors are enabled during writing.
    pub(in crate) color_enabled: bool,
}

impl CodeDisplay {
    /// Constructs a new info-type CodeDisplay with the given description.
    pub fn new<M>(message: M) -> Self 
        where M: Into<String>,
    {
        CodeDisplay {
            message: message.into(),
            message_type: MessageType::Info,
            code_id: None,
            span_displays: Vec::with_capacity(1),
            notes: Vec::new(),
            color_enabled: true,
        }
    }

    /// Returns the given CodeDisplay with the given message.
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    /// Returns the given CodeDisplay with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.color_enabled = color_enabled;
        self
    }


    /// Returns the given CodeDisplay with the error MessageType.
    pub fn with_error_type(mut self) -> Self {
        self.message_type = MessageType::Error;
        self
    }

    /// Returns the given CodeDisplay with the warning MessageType.
    pub fn with_warning_type(mut self) -> Self {
        self.message_type = MessageType::Warning;
        self
    }
    
    /// Returns the given CodeDisplay with the note MessageType.
    pub fn with_note_type(mut self) -> Self {
        self.message_type = MessageType::Note;
        self
    }

    /// Returns the given CodeDisplay with the hel MessageType.
    pub fn with_help_type(mut self) -> Self {
        self.message_type = MessageType::Help;
        self
    }

    /// Returns the given CodeDisplay with the given MessageType.
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Returns the given CodeDisplay with the given error code id.
    pub fn with_code_id(mut self, code_id: Option<&'static str>) -> Self {
        self.code_id = code_id;
        self
    }

    /// Returns the given CodeDisplay with the given SpanDisplay attachment.
    pub fn with_span_display<S>(mut self, span_display: S)
        -> Self
        where S: Into<SpanDisplay>
    {
        self.span_displays.push(span_display.into());
        self
    }

    /// Returns the given CodeDisplay with the given note attachment.
    pub fn with_note<N>(mut self, note: N)
        -> Self
        where N: Into<Note>
    {
        self.notes.push(note.into());
        self
    }

    /// Returns the given CodeDisplay with the given SpanDisplay attachment.
    pub fn push_span_display<S>(&mut self, span_display: S)
        where S: Into<SpanDisplay>
    {
        self.span_displays.push(span_display.into());
    }


    /// Returns the given CodeDisplay with the given note attachment.
    pub fn push_note<N>(&mut self, note: N)
        where N: Into<Note>
    {
        self.notes.push(note.into());
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn write<W>(
        &self,
        out: &mut W,
        source: SourceText<'_>)
        -> std::fmt::Result
        where W: Write
    {
        self.write_with_color_enablement(out, source, self.color_enabled)
    }

    pub(in crate) fn write_with_color_enablement<W>(
        &self,
        out: &mut W,
        source: SourceText<'_>,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        if color_enabled {
            write!(out, "{}", self.message_type)?;
            if let Some(code_id) = self.code_id {
                write!(out, "[{}]", code_id)?;
            }
            writeln!(out, "{} {}",
                ":".bright_white().bold(),
                self.message.bright_white().bold())?;
        } else {
            self.message_type
                .write_with_color_enablement(out, color_enabled)?;
            writeln!(out, ": {}", self.message)?;
        }
        for span_display in &self.span_displays {
            span_display
                .write_with_color_enablement(out, source, color_enabled)?;
        }
        for note in &self.notes {
            note.write_with_color_enablement(out, color_enabled)?;
        }
        Ok(())
    }
}



////////////////////////////////////////////////////////////////////////////////
// SpanDisplay
////////////////////////////////////////////////////////////////////////////////
/// A single span of source text with notes and highlights.
#[derive(Debug, Clone)]
pub struct SpanDisplay {
    /// The name of the file or data that is being displayed.
    pub(in crate) source_name: Option<String>,
    /// The column metrics for the source,
    pub(in crate) metrics: ColumnMetrics,
    /// The full text span of the displayed source.
    pub(in crate) span: Span,
    /// The subsets of the displayed text to highlight.
    pub(in crate) highlights: Vec<Highlight>,
    /// Notes to append to the source display.
    pub(in crate) notes: Vec<Note>,
    /// The width of the line number gutter.
    pub(in crate) gutter_width: u8,
    /// Whether to allow line omissions within the source display.
    pub(in crate) allow_omissions: bool,
}

impl SpanDisplay {
    /// Constructs a new SpanDisplay with the given span.
    pub fn new(source: SourceText<'_>, span: Span) -> Self {
        let gutter_width = std::cmp::max(
            (span.end().page.line as f32).log10().ceil() as u8, 1);

        SpanDisplay {
            source_name: source.name().map(String::from),
            metrics: source.column_metrics(),
            span: span.widen_to_line(source),
            highlights: Vec::with_capacity(2),
            notes: Vec::new(),
            allow_omissions: true,
            gutter_width,
        }
    }

    /// Constructs a new SpanDisplay with the given span and highlight message.
    pub fn new_error_highlight<M>(
        source: SourceText<'_>, 
        span: Span,
        message: M)
        -> Self
        where M: Into<String>,
    {
        SpanDisplay::new(source, span)
            .with_highlight(Highlight::new(span, message)
                .with_error_type())
    }

    /// Returns the given SpanDisplay with the given source name.
    pub fn with_source_name<M>(mut self, name: M) -> Self
        where M: Into<String>,
    {
        self.source_name = Some(name.into());
        self
    }

    /// Attaches the given Highlight to the source span.
    pub fn with_highlight(mut self, highlight: Highlight) -> Self {
        self.highlights.push(highlight);
        self
    }

    /// Attaches the given Note to the source span.
    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    pub(in crate) fn write_with_color_enablement<W>(
        &self,
        out: &mut W,
        source: SourceText<'_>,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        let _span = span!(Level::TRACE, "SpanDisplay", color_enabled).entered();

        let (source_name, sep) = match &self.source_name {
            Some(name) => (name.borrow(), ":"),
            None       => ("", ""),
        };

        if color_enabled {
            writeln!(out, "{:width$}{} {}{}({})",
                "",
                "-->".bright_blue().bold(),
                source_name,
                sep,
                self.span,
                width=self.gutter_width as usize)?;
        } else {
            writeln!(out, "{:width$}--> {}{}({})",
                "",
                source_name,
                sep,
                self.span,
                width=self.gutter_width as usize)?;
        }

        MultiSplitLines::new(
                source,
                self.span,
                &self.highlights[..],
                self.gutter_width)
            .write_with_color_enablement(out, source, color_enabled)?;

        for note in &self.notes {
            write!(out, "{:width$} = ", "", width=self.gutter_width as usize)?;
            note.write_with_color_enablement(out, color_enabled)?;
            writeln!(out)?;
        }

        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////
// Note
////////////////////////////////////////////////////////////////////////////////
/// A note which can be attached to a `SpanDisplay` or `CodeDisplay`.
#[derive(Debug, Clone)]
pub struct Note {
    /// The message type for the note.
    pub(in crate) note_type: MessageType,
    /// The note to display.
    pub(in crate) note: String,
}

impl Note {
    pub(in crate) fn write_with_color_enablement<W>(
        &self,
        out: &mut W,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        self.note_type.write_with_color_enablement(out, color_enabled)?;
        write!(out, ": {}", self.note)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, true)
    }
}



////////////////////////////////////////////////////////////////////////////////
// MultiSplitLines
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the line-based data relevant to a particular SpanDisplay.
#[derive(Debug, Clone)]
struct MultiSplitLines<'text, 'hl> {
    /// The SplitLines iterator for the `SpanDisplay`.
    source_lines: SplitLines<'text>,
    /// The highlights contained within the SpanDisplay.
    highlights: &'hl [Highlight],
    /// The width of the line number gutter.
    gutter_width: u8,
    /// The width of the highlight riser gutter.
    riser_width: u8,
}

impl<'text, 'hl> MultiSplitLines<'text, 'hl>  {
    /// Constructs a new MultiSplitLines from the given source span and
    /// highlights.
    pub(in crate) fn new(
        source: SourceText<'text>,
        span_display: Span,
        highlights: &'hl [Highlight],
        gutter_width: u8)
        -> Self
    {
        let _span = span!(Level::TRACE, "new").entered();

        let riser_width = highlights
            .iter()
            .filter(|h| h.is_multiline())
            .count()
            .try_into()
            .expect("riser width < 255");
        event!(Level::TRACE, "riser_width = {riser_width}");

        let source_lines = span_display.split_lines(source);

        MultiSplitLines {
            source_lines,
            highlights,
            gutter_width,
            riser_width,
        }
    }

    /// Consumes the MultiSplitLines and writes all of the contained data.
    pub(in crate) fn write_with_color_enablement<W>(
        mut self,
        out: &mut W,
        source: SourceText<'text>,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        let _span = span!(Level::TRACE, "MultiSplitLines", color_enabled)
            .entered();

        // Write empty line to uncramp the display.
        write_gutter(out, "", self.gutter_width, color_enabled)?;
        writeln!(out)?;

        let mut riser_states = Vec::with_capacity(self.highlights.len());
        for hl in self.highlights {
            riser_states
                .push(if hl.is_multiline() { 
                    RiserState::Waiting
                } else {
                    RiserState::Unused
                });
        }

        for (i, span) in self.source_lines.enumerate() {
            event!(Level::TRACE, "span {i} = {span}");

            let current_line = span.start().page.line;

            // Write gutter for source line.
            write_gutter(out, current_line, self.gutter_width, color_enabled)?;

            // Write risers for source line.
            let mut multiline_highlights_present = false;
            for (idx, hl) in self.highlights.iter().enumerate() {
                hl.write_riser_for_line(
                    out,
                    current_line,
                    &mut riser_states[idx],
                    false,
                    color_enabled)?;
                if hl.is_multiline() { multiline_highlights_present = true; }
            }
            if multiline_highlights_present { write!(out, " "); }

            // Write source text.
            writeln!(out, "{}", source.clip(span).as_ref())?;

            for (message_idx, message_hl) in self.highlights.iter().enumerate()
            {
                if !message_hl.has_message_for_line(current_line) { continue; }

                // Write message gutter.
                write_gutter(out, "", self.gutter_width, color_enabled)?;
                
                for (idx, hl) in self.highlights.iter().enumerate() {
                    // Write message risers.
                    hl.write_riser_for_line(
                        out,
                        current_line,
                        &mut riser_states[idx],
                        message_idx == idx,
                        color_enabled)?;
                }
                
                // Write message.
                message_hl.write_message_for_line(
                    out,
                    current_line,
                    multiline_highlights_present,
                    color_enabled)?;
            }
        }
        Ok(())
    }
}

fn write_gutter<V, W>(
    out: &mut W,
    value: V,
    width: u8,
    color_enabled: bool)
    -> std::fmt::Result
    where V: Display, W: Write
{
    if color_enabled {
        write!(out, "{:>width$} {} ",
            format!("{}", value).bright_blue().bold(),
            "|".bright_blue().bold(),
            width=width as usize)
    } else {
        write!(out, "{:>width$} | ", value, width=width as usize)
    }
}

fn write_gutter_omit<V, W>(
    out: &mut W,
    value: V,
    width: u8,
    color_enabled: bool)
    -> std::fmt::Result
    where V: Display, W: Write
{
    if color_enabled {
        write!(out, "{:>width$}{}",
            "",
            "...".bright_blue().bold(),
            width=width as usize)
    } else {
        write!(out, "{:>width$}...", "", width=width as usize)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate) enum RiserState {
    Unused,
    Waiting,
    Started,
    Ended,
}
