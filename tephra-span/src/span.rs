////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Text spans.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::position::Pos;
use crate::position::Page;
use crate::position::ColumnMetrics;

// External library imports.
use few::Few;


////////////////////////////////////////////////////////////////////////////////
// Span
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text.
// NOTE: Span methods must maintain an invariant: span.start() < span.end().
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span<'text> {
    /// The source text.
    source: &'text str,
    /// The byte range of the spanned section within the source.
    byte: ByteSpan,
    /// The page range of the spanned section within the source.
    page: PageSpan,
}

impl<'text> Span<'text> {
    /// Constructs a new span covering given source text.
    pub fn full(source: &'text str, metrics: ColumnMetrics)
        -> Self
    {
        Span::new_enclosing(
            source,
            Pos::ZERO,
            metrics.end_position(source, Pos::ZERO))
    }

    /// Constructs a new empty span in the given source text.
    pub fn new(source: &'text str) -> Self {
        Span {
            source,
            byte: ByteSpan::default(),
            page: PageSpan::default(),
        }
    }

    /// Constructs a new empty span in the given source text, starting from the
    /// given byte and page.
    pub fn new_at(source: &'text str, pos: Pos) -> Self {
        Span {
            source,
            byte: ByteSpan { start: pos.byte, end: pos.byte },
            page: PageSpan { start: pos.page, end: pos.page },
        }
    }

    /// Constructs a new span covering given source text.
    pub fn new_enclosing(source: &'text str, a: Pos, b: Pos) -> Self
    {
        Span {
            source,
            byte: ByteSpan { start: a.byte, end: b.byte },
            page: PageSpan { start: a.page, end: b.page },
        }
    }

    /// Returns true if the span is empty
    pub fn is_empty(&self) -> bool {
        self.byte.start == self.byte.end
    }

    /// Returns true if the span covers the entire source text.
    pub fn is_full(&self) -> bool {
        self.byte.start == 0 && self.byte.end == self.source.len()
    }

    /// Returns the spanned text.
    pub fn text(&self) -> &'text str {
        &self.source[self.byte.start..self.byte.end]
    }

    /// Returns the start position of the span.
    pub fn start(&self) -> Pos {
        Pos {
            byte: self.byte.start,
            page: self.page.start,
        }
    }

    /// Returns the end position of the span.
    pub fn end(&self) -> Pos {
        Pos {
            byte: self.byte.end,
            page: self.page.end,
        }
    }

    /// Returns the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.byte.len()
    }

    /// Returns true if the given position is contained within the span.
    ///
    /// This will return true if the position is a boundary point of the span.
    pub fn contains(&self, pos: &Pos) -> bool {
        *pos >= self.start()  && *pos <= self.end()
    }

    /// Widens the span on the left and right to the nearest newline.
    pub fn widen_to_line(&self, metrics: ColumnMetrics)
        -> Self
    {
        if self.is_full() { return self.clone(); }

        Span::new_enclosing(
            self.source,
            metrics.line_start_position(self.source, self.start()),
            metrics.line_end_position(self.source, self.end()))
    }

    /// Returns true if the given spans overlap.
    ///
    /// This will return true if the boundary points of the spans overlap.
    pub fn intersects<S>(&self, other: S) -> bool 
        where S: Into<Self>,
    {
        let other = other.into();
        self.contains(&other.start()) ||
        self.contains(&other.end()) ||
        other.contains(&self.start()) ||
        other.contains(&self.end())
    }

    /// Returns true if the given spans share a boundary point without
    /// containing each other.
    pub fn adjacent<S>(&self, other: S) -> bool 
        where S: Into<Self>,
    {
        let other = other.into();
        self.start() == other.end() || self.end() == other.start()
    }

    /// Returns the smallest span covering the given spans.
    pub fn enclose<S>(&self, other: S) -> Self 
        where S: Into<Self>,
    {
        let other = other.into();
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();
        let start = if a_start < b_start { a_start } else { b_start };
        let end = if a_end > b_end { a_end } else { b_end };
        Span::new_enclosing(self.source, start, end)
    }

    /// Returns the smallest set of spans covering the given spans.
    pub fn union<S>(&self, other: S) -> Few<Self> 
        where S: Into<Self>,
    {
        let other = other.into();
        if self.intersects(other.clone()) {
            Few::One(self.enclose(other))
        } else {
            Few::Two(self.clone(), other)
        }
    }

    /// Returns the overlapping portion the spans.
    pub fn intersect<S>(&self, other: S) -> Option<Self>
        where S: Into<Self>,
    {
        let other = other.into();
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();

        let start = match (self.contains(&b_start), other.contains(&a_start)) {
            (true,  true)  => a_start, // Starts coincide.
            (true,  false) => b_start,
            (false, true)  => a_start,
            (false, false) => return None,
        };

        let end = match (self.contains(&b_end), other.contains(&a_end)) {
            (true,  true)  => a_end, // Ends coincide.
            (true,  false) => b_end,
            (false, true)  => a_end,
            (false, false) => return None,
        };

        Some(Span::new_enclosing(self.source, start, end))
    }

    /// Returns the result of removing a portion of the span.
    ///
    /// Note that if an endpoint becomes an empty span, it is omitted. If the
    /// right span is empty, it effectively splits the left span at that point.
    pub fn minus<S>(&self, other: S) -> Few<Self> 
        where S: Into<Self>,
    {
        let other = other.into();
        let a_0 = self.start();
        let b_0 = other.start();
        let a_1 = self.end();
        let b_1 = other.end();

        let (l, r) = match (a_0 < b_0, a_1 < b_1) {
            (true, true) => (Some((a_0, b_0)), Some((b_1, a_1))),
            (true, _)    => (Some((a_0, b_0)), None),
            (_,    true) => (None,             Some((b_1, a_1))),
            _            => (None,             None),
        };

        let l = l.map(|(a, b)| Span::new_enclosing(self.source, a, b));
        let r = r.map(|(a, b)| Span::new_enclosing(self.source, a, b));
        Few::from((l, r))
    }

    /// Returns an iterator over the lines of the span.
    pub fn split_lines(&self, metrics: ColumnMetrics)
        -> SplitLines<'text>
    {
        SplitLines {
            start: self.start(),
            end: self.end(),
            source: self.source,
            metrics,
        }
    }
}


impl<'text> std::fmt::Display for Span<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.byte.is_empty() {
            write!(f, "{}, byte {}", self.page, self.byte)
        } else {
            write!(f, "{}, bytes {}", self.page, self.byte)
        }
    }
}

impl<'text> std::fmt::Debug for Span<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.byte.is_empty() {
            write!(f,
                "\"{}\" ({}, byte {})", self.text(), self.page, self.byte)
        } else {
            write!(f,
                "\"{}\" ({}, bytes {})", self.text(), self.page, self.byte)
        }
    }
}

impl<'text> From<&'text SpanOwned> for Span<'text> {
    fn from(owned: &'text SpanOwned) -> Self {
        Span {
            source: &*owned.source,
            byte: owned.byte,
            page: owned.page,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ByteSpan
////////////////////////////////////////////////////////////////////////////////
/// The interval of bytes which span the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteSpan {
    // NOTE: Field order must be maintained for PartialOrd and Ord impls.
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
}

impl ByteSpan {
    /// Returns the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Default for ByteSpan {
    fn default() -> Self {
        ByteSpan { start: 0, end: 0 }
    }
}

impl std::fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// PageSpan
////////////////////////////////////////////////////////////////////////////////
/// The interval of lines and columns which span the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PageSpan {
    // NOTE: Field order must be maintained for PartialOrd and Ord impls.
    /// The start of the span.
    pub start: Page,
    /// The end of the span.
    pub end: Page,
}

impl PageSpan {
    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Default for PageSpan {
    fn default() -> Self {
        PageSpan { start: Page::ZERO, end: Page::ZERO }
    }
}

impl std::fmt::Display for PageSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// SpanOwned
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text.
// NOTE: Span methods must maintain an invariant: span.start() < span.end().
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanOwned {
    /// The source text.
    source: Box<str>,
    /// The byte range of the captured section within the source.
    full_byte: ByteSpan,
    /// The page range of the captured section within the source.
    full_page: PageSpan,
    /// The byte range of the spanned section within the source.
    byte: ByteSpan,
    /// The page range of the spanned section within the source.
    page: PageSpan,
}


impl SpanOwned {

    /// Returns the spanned text.
    pub fn text(&self) -> &str {
        &self.source[self.byte.start..self.byte.end]
    }

    /// Returns the start position of the span.
    pub fn start(&self) -> Pos {
        Pos {
            byte: self.byte.start,
            page: self.page.start,
        }
    }

    /// Returns the end position of the span.
    pub fn end(&self) -> Pos {
        Pos {
            byte: self.byte.end,
            page: self.page.end,
        }
    }

    /// Returns the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.byte.len()
    }

    /// Returns true if the given position is contained within the span.
    ///
    /// This will return true if the position is a boundary point of the span.
    pub fn contains(&self, pos: &Pos) -> bool {
        *pos >= self.start()  && *pos <= self.end()
    }

    /// Returns true if the given spans overlap.
    ///
    /// This will return true if the boundary points of the spans overlap.
    pub fn intersects(&self, other: &Self) -> bool {
        self.contains(&other.start()) ||
        self.contains(&other.end()) ||
        other.contains(&self.start()) ||
        other.contains(&self.end())
    }

    /// Returns true if the given spans share a boundary point without
    /// containing each other.
    pub fn adjacent(&self, other: &Self) -> bool {
        self.start() == other.end() || self.end() == other.start()
    }

    /// Returns the overlapping portion the spans.
    pub fn intersect<'text, S>(&'text self, other: S) -> Option<Span<'text>> 
        where S: Into<Span<'text>>,
    {
        let other = other.into();
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();

        let start = match (self.contains(&b_start), other.contains(&a_start)) {
            (true,  true)  => a_start, // Starts coincide.
            (true,  false) => b_start,
            (false, true)  => a_start,
            (false, false) => return None,
        };

        let end = match (self.contains(&b_end), other.contains(&a_end)) {
            (true,  true)  => a_end, // Ends coincide.
            (true,  false) => b_end,
            (false, true)  => a_end,
            (false, false) => return None,
        };

        Some(Span::new_enclosing(other.source, start, end))
    }

    /// Returns the result of removing a portion of the span.
    ///
    /// Note that if an endpoint becomes an empty span, it is omitted. If the
    /// right span is empty, it effectively splits the left span at that point.
    pub fn minus<'text, S>(&'text self, other: S) -> Few<Span<'text>> 
        where S: Into<Span<'text>>,
    {
        let other = other.into();
        let a_0 = self.start();
        let b_0 = other.start();
        let a_1 = self.end();
        let b_1 = other.end();

        let (l, r) = match (a_0 < b_0, a_1 < b_1) {
            (true, true) => (Some((a_0, b_0)), Some((b_1, a_1))),
            (true, _)    => (Some((a_0, b_0)), None),
            (_,    true) => (None,             Some((b_1, a_1))),
            _            => (None,             None),
        };

        let l = l.map(|(a, b)| Span::new_enclosing(other.source, a, b));
        let r = r.map(|(a, b)| Span::new_enclosing(other.source, a, b));
        Few::from((l, r))
    }
}

impl std::fmt::Display for SpanOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({}, bytes {})", self.text(), self.page, self.byte)
    }
}

impl<'text> From<Span<'text>> for SpanOwned {
    fn from(span: Span<'text>) -> Self {
        let full_span = Span::new_at(span.source, Pos::ZERO);
        SpanOwned {
            source: span.source.to_owned().into(),
            full_byte: full_span.byte,
            full_page: full_span.page,
            byte: span.byte,
            page: span.page,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// SplitLines
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the lines of a span. Returned by the `lines` method on
/// `Span`.
#[derive(Debug, Clone)]
pub struct SplitLines<'text> {
    source: &'text str,
    start: Pos,
    end: Pos,
    metrics: ColumnMetrics,
}

impl<'text> Iterator for SplitLines<'text> {
    type Item = Span<'text>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.start.page.line > self.end.page.line {
            None
        } else if self.start.page.line == self.end.page.line {
            // Last line; no need to advance the start position.
            let res = Some(Span::new_enclosing(
                self.source,
                self.start,
                self.end));

            self.start.page.line += 1;
            res
        } else {
            let end = self.metrics
                .line_end_position(self.source, self.start);

            let res = Some(Span::new_enclosing(
                self.source,
                self.start,
                end));

            self.start = self.metrics
                .next_position(self.source, end)
                .expect("next line < end line");
            res
        }
    }
}

impl<'text> std::iter::FusedIterator for SplitLines<'text> {}

impl<'text> ExactSizeIterator for SplitLines<'text> {
    fn len(&self) -> usize {
         self.end.page.line - self.start.page.line
    }
}

