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
    start: Pos,
}

impl<'text> SourceText<'text> {
    /// Constructs a new `SourceText` with the given start `Pos` and
    /// `ColumnMetrics`.
    pub fn new(source: &'text str, start: Pos, metrics: ColumnMetrics) -> Self {
        SourceText {
            source,
            start,
            metrics,
        }
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn text(&self) -> &'text str {
        &self.source
    }

    pub fn start(&self) -> Pos {
        self.start
    }

    pub fn start_mut(&mut self) -> &mut Pos {
        &mut self.start
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        &mut self.metrics
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

    /// Returns the start position of the text, given its end position.
    pub fn start_position(&self, end: Pos) -> Pos {
        self.metrics.start_position(self.source, end)
    }

    /// Returns the end position of the text, given its start position.
    pub fn end_position(&self, start: Pos) -> Pos {
        self.metrics.end_position(self.source, start)
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
            start: self.start,
            metrics: self.metrics,
        }
    }
}


impl<'text> std::fmt::Display for SourceText<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.source.len() > 7 {
            write!(f, "{}...", &self.source[0..7])?;
        } else {
            write!(f, "{}...", &self.source[..])?;
        };

        write!(f, " ({}, {:?})", self.start, self.metrics)
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
            .field("start", &self.start)
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
    start: Pos,
}

impl SourceTextOwned {
    /// Constructs a new `SourceTextOwned` with the given start `Pos` and
    /// `ColumnMetrics`.
    pub fn new(source: Box<str>, start: Pos, metrics: ColumnMetrics) -> Self {
        SourceTextOwned {
            source,
            start,
            metrics,
        }
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn text(&self) -> &str {
        self.source.as_ref()
    }

    pub fn start(&self) -> Pos {
        self.start
    }

    pub fn start_mut(&mut self) -> &mut Pos {
        &mut self.start
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        &mut self.metrics
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

    /// Returns the start position of the text, given its end position.
    pub fn start_position(&self, end: Pos) -> Pos {
        self.metrics.start_position(&self.source, end)
    }

    /// Returns the end position of the text, given its start position.
    pub fn end_position(&self, start: Pos) -> Pos {
        self.metrics.end_position(&self.source, start)
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
            start: self.start,
            metrics: self.metrics,
        }
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








