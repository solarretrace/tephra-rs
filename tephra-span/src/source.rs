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
use crate::Span;

// External library imports.
use tephra_tracing::span;
use tephra_tracing::Level;


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
    pub fn empty() -> SourceText<'static> {
        SourceText::new("")
    }

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

    pub fn as_str(&self) -> &'text str {
        &self.source
    }

    #[inline(always)]
    fn pos_in_bounds(&self, pos: Pos) -> bool {
        let end = self.end_position();
        pos.byte >= self.offset.byte
            && pos.byte <= end.byte
            && pos.page >= self.offset.page 
            && pos.page <= end.page 
    }

    pub fn clip(&self, span: Span) -> Self {
        debug_assert!(self.pos_in_bounds(span.start()),
            "start of span is out of source text bounds");
        debug_assert!(self.pos_in_bounds(span.end()),
            "start of span is out of source text bounds");

        let s = span.start().byte - self.offset.byte;
        let e = span.end().byte - self.offset.byte;
        SourceText {
            source: &self.source[s..e],
            metrics: self.metrics,
            offset: span.start(),
        }
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        &mut self.metrics
    }

    /// Returns the full span of the text.
    pub fn full_span(&self) -> Span {
        let end = self.end_position();
        Span::new_enclosing(self.offset, end)
    }

    /// Returns the start position of the text.
    pub fn start_position(&self) -> Pos {
        self.offset
    }

    /// Returns the end position of the text.
    pub fn end_position(&self) -> Pos {
        let end = self.metrics.end_position(&self.source, Pos::ZERO);
        self.offset.shifted(end)
    }

    /// Returns the next column-aligned position after the given base position
    /// within the source text. None is returned if the result position is not
    /// within the text.
    pub fn next_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_position(self.source, b))
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the source text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_position(self.source, b))
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break(&self, byte: usize) -> bool {
        debug_assert!(byte > self.offset.byte,
            "byte is out of source text bounds");
        self.metrics.is_line_break(self.source, byte - self.offset.byte)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_end_position(self.source, b)))
            .unwrap()
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    pub fn line_start_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_start_position(self.source, b)))
            .unwrap()
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_line_end_position(self.source, b))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_line_start_position(self.source, b))
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    pub fn position_after_str<'a>(&self, start: Pos, pattern: &'a str)
        -> Option<Pos>
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_str(self.source, s, pattern))
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_chars_matching(self.source, s, f))
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte, |s| self
            .metrics
            .next_position_after_chars_matching(self.source, s, f))
    }

    /// Returns an iterator over the display columns of the source text.
    pub fn iter_columns(&self, base: Pos) -> IterColumns<'text> {
        IterColumns {
            source: self.source,
            metrics: self.metrics,
            offset: self.offset,
            base: Some(base),
        }
    }

    /// Converts the `SourceText` into a `SourceTextOwned` by cloning the
    /// backing text buffer.
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
    pub fn empty() -> Self {
        SourceTextOwned::new("")
    }

    /// Constructs a new `SourceText` with the given offset `Pos` and
    /// `ColumnMetrics`.
    pub fn new(source: &str) -> Self {
        SourceTextOwned {
            source: source.into(),
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

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }
    
    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        &mut self.metrics
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.source
    }

    pub fn clip<'text>(&'text self, span: Span) -> SourceText<'text> {
        self.as_borrowed().clip(span)
    }

    /// Returns the full span of the text.
    pub fn full_span(&self) -> Span {
        let end = self.end_position();
        Span::new_enclosing(self.offset, end)
    }

    /// Returns the start position of the text.
    pub fn start_position(&self) -> Pos {
        self.offset
    }

    /// Returns the end position of the text.
    pub fn end_position(&self) -> Pos {
        let end = self.metrics.end_position(&self.source, Pos::ZERO);
        self.offset.shifted(end)
    }

    /// Returns the next column-aligned position after the given base position
    /// within the source text. None is returned if the result position is not
    /// within the text.
    pub fn next_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_position(&self.source, b))
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the source text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_position(&self.source, b))
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break(&self, byte: usize) -> bool {
        debug_assert!(byte > self.offset.byte,
            "byte is out of source text bounds");
        self.metrics.is_line_break(&self.source, byte - self.offset.byte)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_end_position(&self.source, b)))
            .unwrap()
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    pub fn line_start_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_start_position(&self.source, b)))
            .unwrap()
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_line_end_position(&self.source, b))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_line_start_position(&self.source, b))
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    pub fn position_after_str<'a>(&self, start: Pos, pattern: &'a str)
        -> Option<Pos>
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_str(&self.source, s, pattern))
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_chars_matching(&self.source, s, f))
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte, |s| self
            .metrics
            .next_position_after_chars_matching(&self.source, s, f))
    }

    /// Returns an iterator over the display columns of the source text.
    pub fn iter_columns<'text>(&'text self, base: Pos) -> IterColumns<'text> {
        IterColumns {
            source: &self.source,
            metrics: self.metrics,
            offset: self.offset,
            base: Some(base),
        }
    }

    /// Returns a `SourceText` that borrows from the `SourceTextOwned`.
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




////////////////////////////////////////////////////////////////////////////////
// IterColumns
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the display columns of a source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IterColumns<'text> {
    /// The source text.
    source: &'text str,
    /// The column metrics of the source text.
    metrics: ColumnMetrics,
    /// The position of the start of the source text.
    offset: Pos,
    /// The current column position.
    base: Option<Pos>,
}

impl<'text> Iterator for IterColumns<'text> {
    type Item = (&'text str, Pos);

    fn next(&mut self) -> Option<Self::Item> {
        let _span = span!(Level::TRACE, "IterColumns::next").entered();

        // TODO: handle offsets.
        let end = self.base
            .and_then(|start| self.metrics.next_position(self.source, start));

        let res = match (self.base, end) {
            (Some(s), Some(e)) => Some((&self.source[s.byte..e.byte], e)),
            _                  => None,
        };

        self.base = end;
        res
    }
}

impl<'text> std::iter::FusedIterator for IterColumns<'text> {}
