////////////////////////////////////////////////////////////////////////////////
// Tephra parser span library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Source text wrapper.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::ColumnMetrics;
use crate::Pos;
use crate::IterColumns;
use crate::Span;


////////////////////////////////////////////////////////////////////////////////
// SourceText
////////////////////////////////////////////////////////////////////////////////
/// A positioned section of source text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceText<'text> {
    /// The source text.
    source: &'text str,
    /// The column metrics of the source text.
    metrics: ColumnMetrics,
    /// The position of the start of the source text.
    offset: Pos,
}

impl<'text> SourceText<'text> {
    /// Constructs a new `SourceText` with the given offset `Pos` and
    /// `ColumnMetrics`.
    pub fn new(source: &'text str) -> Self {
        SourceText {
            source,
            offset: Pos::ZERO,
            metrics: ColumnMetrics::default(),
        }
    }

    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_start_position(mut self, offset: Pos) -> Self {
        self.offset = offset;
        self
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn clip(&self, span: Span) -> Self {
        SourceText {
            source: &self.source[span.start().byte..span.end().byte],
            metrics: self.metrics,
            offset: span.start(),
        }
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    /// Returns the full span of the text.
    pub fn full_span(&self) -> Span {
        let end = self.metrics.end_position(self.source, self.offset);
        Span::new_enclosing(self.offset, end)
    }

    /// Returns the start position of the text.
    pub fn start_position(&self) -> Pos {
        self.offset
    }

    /// Returns the end position of the text.
    pub fn end_position(&self) -> Pos {
        self.metrics.end_position(&self.source, self.offset)
    }

    /// Returns the next column-aligned position after the given base position
    /// within the source text. None is returned if the result position is not
    /// within the text.
    pub fn next_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.next_position(self.source, base)
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the source text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.previous_position(self.source, base)
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break(&self, byte: usize) -> bool {
        self.metrics.is_line_break(self.source, byte)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position(&self, base: Pos) -> Pos {
        self.metrics.line_end_position(self.source, base)
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    pub fn line_start_position(&self, base: Pos) -> Pos {
        self.metrics.line_start_position(self.source, base)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.previous_line_end_position(self.source, base)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.next_line_start_position(self.source, base)
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    pub fn position_after_str<'a>(&self, start: Pos, pattern: &'a str)
        -> Option<Pos>
    {
        self.metrics.position_after_str(self.source, start, pattern)
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        self.metrics.position_after_chars_matching(self.source, start, f)
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        self.metrics.next_position_after_chars_matching(self.source, start, f)
    }

    /// Returns an iterator over the display columns of the source text.
    pub fn iter_columns(&self, base: Pos) -> IterColumns<'text> {
        self.metrics.iter_columns(self.source, base)
    }

    pub fn to_owned(&self) -> SourceTextOwned {
        SourceTextOwned {
            source: self.source.into(),
            offset: self.offset,
            metrics: self.metrics,
        }
    }
}

impl<'text> AsRef<str> for SourceText<'text> {
    fn as_ref(&self) -> &str {
        &self.source
    }
}

impl<'text> std::fmt::Display for SourceText<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.source.len() > 7 {
            write!(f, "{}...", &self.source[0..7])?;
        } else {
            write!(f, "{}...", &self.source[..])?;
        };

        write!(f, " ({}, {:?})", self.offset, self.metrics)
    }
}

impl<'text> std::fmt::Debug for SourceText<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = if self.source.len() > 7 {
            format!("{}...", &self.source[0..7])
        } else {
            format!("{}...", &self.source[..])
        };
        f.debug_struct("SourceText")
            .field("source", &src)
            .field("offset", &self.offset)
            .field("metrics", &self.metrics)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// SourceTextOwned
////////////////////////////////////////////////////////////////////////////////
/// A positioned section of (owned) source text.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SourceTextOwned {
    /// The source text.
    source: Box<str>,
    /// The column metrics of the source text.
    metrics: ColumnMetrics,
    /// The position of the start of the source text.
    offset: Pos,
}

impl SourceTextOwned {
    /// Constructs a new `SourceTextOwned` with the given offset `Pos` and
    /// `ColumnMetrics`.
    pub fn new(source: Box<str>, offset: Pos, metrics: ColumnMetrics) -> Self {
        SourceTextOwned {
            source,
            offset,
            metrics,
        }
    }

    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_start_position(mut self, offset: Pos) -> Self {
        self.offset = offset;
        self
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn clip<'text>(&'text self, span: Span) -> SourceText<'text> {
        self.as_borrowed().clip(span)
    }

    /// Returns the full span of the text.
    pub fn full_span(&self) -> Span {
        let end = self.metrics.end_position(&self.source, self.offset);
        Span::new_enclosing(self.offset, end)
    }

    /// Returns the start position of the text.
    pub fn start_position(&self) -> Pos {
        self.offset
    }

    /// Returns the end position of the text.
    pub fn end_position(&self) -> Pos {
        self.metrics.end_position(&self.source, self.offset)
    }

    /// Returns the next column-aligned position after the given base position
    /// within the source text. None is returned if the result position is not
    /// within the text.
    pub fn next_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.next_position(&self.source, base)
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the source text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.previous_position(&self.source, base)
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break(&self, byte: usize) -> bool {
        self.metrics.is_line_break(&self.source, byte)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position(&self, base: Pos) -> Pos {
        self.metrics.line_end_position(&self.source, base)
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    pub fn line_start_position(&self, base: Pos) -> Pos {
        self.metrics.line_start_position(&self.source, base)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.previous_line_end_position(&self.source, base)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position(&self, base: Pos) -> Option<Pos> {
        self.metrics.next_line_start_position(&self.source, base)
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    pub fn position_after_str<'a>(&self, start: Pos, pattern: &'a str)
        -> Option<Pos>
    {
        self.metrics.position_after_str(&self.source, start, pattern)
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        self.metrics.position_after_chars_matching(&self.source, start, f)
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        self.metrics.next_position_after_chars_matching(&self.source, start, f)
    }

    /// Returns an iterator over the display columns of the source text.
    pub fn iter_columns<'text>(&'text self, base: Pos) -> IterColumns<'text> {
        self.metrics.iter_columns(&self.source, base)
    }

    pub fn as_borrowed<'text>(&'text self) -> SourceText<'text> {
        SourceText {
            source: &self.source,
            offset: self.offset,
            metrics: self.metrics,
        }
    }
}

impl AsRef<str> for SourceTextOwned {
    fn as_ref(&self) -> &str {
        &self.source
    }
}

impl std::fmt::Display for SourceTextOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_borrowed())
    }
}

impl std::fmt::Debug for SourceTextOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_borrowed())
    }
}
