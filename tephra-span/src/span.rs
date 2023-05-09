////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Text spans.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Page;
use crate::Pos;
use crate::SourceTextRef;

// External library imports.
use few::Few;

// Standard library imports.
use std::cmp::Ordering;

////////////////////////////////////////////////////////////////////////////////
// Span
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text.
// NOTE: Span methods must maintain an invariant: span.start() < span.end().
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Span {
    /// The byte range of the spanned section within the source.
    byte: ByteSpan,
    /// The page range of the spanned section within the source.
    page: PageSpan,
}

impl Span {
    /// Constructs a new empty span.
    #[must_use]
    pub fn new() -> Self {
        Self {
            byte: ByteSpan::default(),
            page: PageSpan::default(),
        }
    }

    /// Constructs a new empty span starting from the given byte and page.
    #[must_use]
    pub fn new_at(pos: Pos) -> Self {
        let byte = ByteSpan { start: pos.byte, end: pos.byte };
        let page = PageSpan { start: pos.page, end: pos.page };

        Self { byte, page }
    }

    /// Constructs a new span covering given start and end positions.
    #[must_use]
    pub fn new_enclosing(mut a: Pos, mut b: Pos) -> Self {
        if a.byte > b.byte { std::mem::swap(&mut a, &mut b); }
        let byte = ByteSpan { start: a.byte, end: b.byte };
        let page = PageSpan { start: a.page, end: b.page };

        Self { byte, page }
    }

    /// Returns true if the span is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.byte.start == self.byte.end
    }

    /// Returns true if the span covers the entire source text.
    #[must_use]
    pub fn is_full(&self, source: SourceTextRef<'_>) -> bool {
        (self.byte.end - self.byte.start) == source.len()
    }

    /// Returns the start position of the span.
    #[must_use]
    pub fn start(&self) -> Pos {
        Pos {
            byte: self.byte.start,
            page: self.page.start,
        }
    }

    /// Returns the end position of the span.
    #[must_use]
    pub fn end(&self) -> Pos {
        Pos {
            byte: self.byte.end,
            page: self.page.end,
        }
    }

    /// Returns the `ByteSpan` of the span.
    #[must_use]
    pub fn byte_span(&self) -> ByteSpan {
        self.byte
    }
    /// Returns the `PageSpan` of the span.
    #[must_use]
    pub fn page_span(&self) -> PageSpan {
        self.page
    }

    /// Returns the length of the span in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.byte.len()
    }

    /// Returns true if the given position is contained within the span.
    ///
    /// This will return true if the position is a boundary point of the span.
    #[must_use]
    pub fn contains(&self, pos: &Pos) -> bool {
        *pos >= self.start()  && *pos <= self.end()
    }

    /// Widens the span on the left and right to the nearest newline.
    #[must_use]
    pub fn widen_to_line(&self, source: SourceTextRef<'_>) -> Self {
        if self.is_full(source) {
            return *self;
        }

        Self::new_enclosing(
            source.line_start_position(self.start()),
            source.line_end_position(self.end()))
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
    #[must_use]
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
        Self::new_enclosing(start, end)
    }

    /// Returns the smallest set of spans covering the given spans.
    #[must_use]
    pub fn union<S>(&self, other: S) -> Few<Self> 
        where S: Into<Self>,
    {
        let other = other.into();
        if self.intersects(other) {
            Few::One(self.enclose(other))
        } else {
            Few::Two(*self, other)
        }
    }

    /// Returns the overlapping portion the spans.
    #[allow(clippy::match_same_arms)]
    #[must_use]
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

        Some(Self::new_enclosing(start, end))
    }

    /// Returns the result of removing a portion of the span.
    ///
    /// Note that if an endpoint becomes an empty span, it is omitted. If the
    /// right span is empty, it effectively splits the left span at that point.
    #[must_use]
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

        let l = l.map(|(a, b)| Self::new_enclosing(a, b));
        let r = r.map(|(a, b)| Self::new_enclosing(a, b));
        Few::from((l, r))
    }

    /// Returns an iterator over the lines of the span.
    #[must_use]
    pub fn split_lines<'text>(&self, source: SourceTextRef<'text>)
        -> SplitLines<'text>
    {
        SplitLines {
            start: self.start(),
            end: self.end(),
            source,
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.byte.is_empty() {
            write!(f, "{}, byte {}", self.page, self.byte)
        } else {
            write!(f, "{}, bytes {}", self.page, self.byte)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ByteSpan
////////////////////////////////////////////////////////////////////////////////
/// The interval of bytes which span the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ByteSpan {
    // NOTE: Field order must be maintained for PartialOrd and Ord impls.
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
}

impl ByteSpan {
    /// Returns the length of the span in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
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

impl Default for PageSpan {
    fn default() -> Self {
        Self { start: Page::ZERO, end: Page::ZERO }
    }
}

impl PageSpan {
    /// Returns true if the span is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
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
// SplitLines
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the lines of a span. Returned by the `lines` method on
/// `Span`.
#[derive(Debug, Clone)]
pub struct SplitLines<'text> {
    /// The source text to split.
    source: SourceTextRef<'text>,
    /// The position to start the split within the source text.
    start: Pos,
    /// The position to end the split within the source text.
    end: Pos,
}

impl<'text> Iterator for SplitLines<'text> {
    type Item = Span;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.start.page.line.cmp(&self.end.page.line) {
            Ordering::Greater => None,

            Ordering::Equal => {
                // Last line; no need to advance the start position.
                let res = Some(Span::new_enclosing(
                    self.start,
                    self.end));

                self.start.page.line += 1;

                res
            },
            Ordering::Less => {
                let end = self.source
                    .line_end_position(self.start);

                let res = Some(Span::new_enclosing(
                    self.start,
                    end));

                self.start = self.source
                    .next_position(end)
                    .expect("next line < end line");

                res
            },
        }
    }
}

impl<'text> std::iter::FusedIterator for SplitLines<'text> {}

impl<'text> ExactSizeIterator for SplitLines<'text> {
    fn len(&self) -> usize {
         self.end.page.line - self.start.page.line
    }
}

