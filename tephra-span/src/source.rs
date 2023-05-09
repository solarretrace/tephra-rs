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


pub const SOURCE_TEXT_DISPLAY_LEN: usize = 12;
pub const SOURCE_TEXT_DEBUG_LEN: usize = 12;

////////////////////////////////////////////////////////////////////////////////
// SourceText
////////////////////////////////////////////////////////////////////////////////

pub type SourceTextRef<'text> = SourceText<&'text str>;
pub type SourceTextOwned = SourceText<Box<str>>;


/// A positioned section of source text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceText<T> {
    /// The source text.
    text: T,
    /// The source text name.
    name: Option<T>,
    /// The column metrics of the source text.
    metrics: ColumnMetrics,
    /// The position of the start of the source text.
    offset: Pos,
}

impl<T> SourceText<T> where T: Default {
    #[must_use]
    pub fn empty() -> Self{
        Self::new(T::default())
    }
}

impl<T> SourceText<T> {
    /// Constructs a new `SourceText` with the given offset `Pos` and
    /// `ColumnMetrics`.
    #[must_use]
    pub fn new(text: T) -> Self {
        Self {
            text,
            name: None,
            offset: Pos::ZERO,
            metrics: ColumnMetrics::default(),
        }
    }

    pub fn text(&self) -> &T {
        &self.text
    }

    #[must_use]
    pub fn with_name(mut self, name: T) -> Self {
        self.name = Some(name);
        self
    }

    #[must_use]
    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
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
}

impl<T> SourceText<T> where T: AsRef<str> {
    pub fn as_str(&self) -> &'_ str {
        self.text.as_ref()
    }

    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    pub fn name(&self) -> Option<&'_ str> {
        self.name.as_ref().map(std::convert::AsRef::as_ref)
    }

    #[inline(always)]
    fn pos_in_bounds(&self, pos: Pos) -> bool {
        let end = self.end_position();
        pos.byte >= self.offset.byte
            && pos.byte <= end.byte
            && pos.page >= self.offset.page 
            && pos.page <= end.page 
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
        let end = self.metrics.end_position(self.as_str(), Pos::ZERO);
        self.offset.shifted(end)
    }

    /// Returns the next column-aligned position after the given base position
    /// within the source text. None is returned if the result position is not
    /// within the text.
    pub fn next_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_position(self.as_str(), b))
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the source text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_position(self.as_str(), b))
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break(&self, byte: usize) -> bool {
        debug_assert!(byte > self.offset.byte,
            "byte is out of source text bounds");
        self.metrics.is_line_break(self.as_str(), byte - self.offset.byte)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_end_position(self.as_str(), b)))
            .unwrap()
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    pub fn line_start_position(&self, base: Pos) -> Pos {
        base.with_byte_offset(self.offset.byte,
                |b| Some(self.metrics.line_start_position(self.as_str(), b)))
            .unwrap()
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.previous_line_end_position(self.as_str(), b))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position(&self, base: Pos) -> Option<Pos> {
        base.with_byte_offset(self.offset.byte,
            |b| self.metrics.next_line_start_position(self.as_str(), b))
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    pub fn position_after_str(&self, start: Pos, pattern: &str)
        -> Option<Pos>
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_str(self.as_str(), s, pattern))
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte,
            |s| self.metrics.position_after_chars_matching(self.as_str(), s, f))
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<F>(&self, start: Pos, f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        start.with_byte_offset(self.offset.byte, |s| self
            .metrics
            .next_position_after_chars_matching(self.as_str(), s, f))
    }

    /// Returns an iterator over the display columns of the source text.
    pub fn iter_columns(&self, base: Pos) -> IterColumns<'_> {
        IterColumns {
            text: self.as_str(),
            metrics: self.metrics,
            offset: self.offset,
            base: Some(base),
        }
    }

    pub fn borrow(&self) -> SourceTextRef<'_> {
        SourceText {
            text: self.text.as_ref(),
            name: self.name(),
            offset: self.offset,
            metrics: self.metrics,
        }
    }

    /// Converts the `SourceText` into a `SourceTextOwned` by cloning the
    /// backing text buffer.
    pub fn to_owned(&self) -> SourceTextOwned {
        SourceText {
            text: self.as_str().into(),
            name: self.name.as_ref().map(|s| s.as_ref().into()),
            offset: self.offset,
            metrics: self.metrics,
        }
    }
}

impl<'text, T> SourceText<T>
    where T: AsRef<str> + From<&'text str> + 'text
{
    #[must_use]
    pub fn clipped(&'text self, span: Span) -> Self {
        debug_assert!(self.pos_in_bounds(span.start()),
            "start of span is out of source text bounds");
        debug_assert!(self.pos_in_bounds(span.end()),
            "start of span is out of source text bounds");

        let s = span.start().byte - self.offset.byte;
        let e = span.end().byte - self.offset.byte;
        Self {
            text: T::from(&self.as_str()[s..e]),
            name: self.name.as_ref().map(|n| T::from(n.as_ref())),
            metrics: self.metrics,
            offset: span.start(),
        }
    }
}


impl<T> AsRef<str> for SourceText<T> where T: AsRef<str> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}


impl<T> std::fmt::Display for SourceText<T> where T: AsRef<str> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.as_str();

        if text.len() > SOURCE_TEXT_DISPLAY_LEN {
            write!(f, "{}...", &text[0..SOURCE_TEXT_DISPLAY_LEN])?;
        } else {
            write!(f, "{text}")?;
        };

        write!(f, " ({}, {:?})", self.offset, self.metrics)
    }
}

impl<T> std::fmt::Debug for SourceText<T>  where T: AsRef<str> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.as_str();
        let src = if text.len() > SOURCE_TEXT_DEBUG_LEN {
            format!("{}...", &text[0..SOURCE_TEXT_DEBUG_LEN])
        } else {
            format!("{text}")
        };

        f.debug_struct("SourceText")
            .field("text", &src)
            .field("name", &self.name.as_ref().map(std::convert::AsRef::as_ref))
            .field("offset", &self.offset)
            .field("metrics", &self.metrics)
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// IterColumns
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the display columns of a source text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IterColumns<'text> {
    /// The source text.
    text: &'text str,
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
        // TODO: handle offsets.
        let end = self.base
            .and_then(|start| self.metrics.next_position(self.text, start));

        let res = match (self.base, end) {
            (Some(s), Some(e)) => Some((&self.text[s.byte..e.byte], e)),
            _                  => None,
        };

        self.base = end;
        res
    }
}

impl<'text> std::iter::FusedIterator for IterColumns<'text> {}
