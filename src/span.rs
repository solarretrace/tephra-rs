////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Text spans.
////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////
// OwnedSpan
////////////////////////////////////////////////////////////////////////////////
/// A owned section of source text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedSpan {
    /// The spanned text, detached from the source.
    pub text: Box<str>,
    /// The byte range of the spanned text within the source.
    pub byte: ByteSpan,
    /// The page range of the spanned text within the source.
    pub page: PageSpan,
}

impl PartialOrd for OwnedSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.byte.partial_cmp(&other.byte)
    }
}

impl std::fmt::Display for OwnedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({}, bytes {})", self.text, self.page, self.byte)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Pos
////////////////////////////////////////////////////////////////////////////////
/// A span relative to an untracked previous position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    /// The relative byte position.
    pub byte: usize,
    /// The relative page position.
    pub page: Page,
}

impl Pos {
    /// The initial span position.
    pub const ZERO: Pos = Pos { byte: 0, page: Page::ZERO };

    /// Creates a Pos with the given number of bytes, lines, and
    /// columns.
    pub fn new(byte: usize, line: usize, column: usize) -> Self {
        Pos {
            byte,
            page: Page { line, column },
        }
    }

    /// Return true if the span position is the zero position.
    pub fn is_zero(self) -> bool {
        self == Pos::ZERO
    }

    /// Increments the Pos by the given number of bytes, lines, and
    /// columns.
    pub fn step(&mut self, bytes: usize, lines: usize, columns: usize) {
        self.byte += bytes;
        self.page.line += lines;
        if lines == 0 {
            self.page.column += columns;
        } else {
            self.page.column = columns;
        }
    }

    /// Increments the Pos by the given position value.
    pub fn step_with(&mut self, pos: Pos) {
        self.step(pos.byte, pos.page.line, pos.page.column)
    }


    /// Constructs the end position from the given string.
    pub fn new_from_string<S, N>(text: S, newline: N) -> Self
        where
            S: AsRef<str>,
            N: AsRef<str>,
    {
        let text = text.as_ref();
        Pos {
            byte: text.len(),
            page: Page::ZERO.advance(text, newline),
        }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Pos::ZERO
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, byte {}", self.page, self.byte)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Span
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord)]
pub struct Span<'text> {
    /// The source text.
    pub source: &'text str,
    /// The byte range of the spanned section within the source.
    pub byte: ByteSpan,
    /// The page range of the spanned section within the source.
    pub page: PageSpan,
}

impl<'text> Span<'text> {
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
    pub fn new_from(pos: Pos, source: &'text str) -> Self {
        Span {
            source,
            byte: ByteSpan { start: pos.byte, end: pos.byte },
            page: PageSpan { start: pos.page, end: pos.page },
        }
    }

    /// Extends the span by the given span position.
    pub fn extend_by(&mut self, pos: Pos) {
        self.byte.end += pos.byte;
        if pos.page.line == 0 {
            self.page.end = Page {
                line: self.page.end.line,
                column: self.page.end.column + pos.page.column,
            };
        } else {
            self.page.end = Page {
                line: self.page.end.line + pos.page.line,
                column: pos.page.column,
            };
        }
    }

    /// Extends the span by the given number of bytes.
    pub fn extend_by_bytes<N>(&mut self, bytes: usize, newline: N)
        where N: AsRef<str>
    {
        let substr = &self.source[self.byte.end..self.byte.end+bytes];

        self.byte = ByteSpan::from_text_starting(self.byte.end, substr);
        self.page = PageSpan::from_text_starting(
            self.page.start,
            substr,
            newline);
    }

    /// Converts the span into and OwnedSpan.
    pub fn to_owned_span(self) -> OwnedSpan {
        OwnedSpan {
            text: self.source.to_owned().into(),
            byte: self.byte,
            page: self.page,
        }
    }

    /// Returns the spanned text.
    pub fn text(&self) -> &'text str {
        &self.source[self.byte.start..self.byte.end]
    }

    /// Returns the start position of the span.
    pub fn start_position(&self) -> Pos {
        Pos {
            byte: self.byte.start,
            page: self.page.start,
        }
    }

    /// Returns the end position of the span.
    pub fn end_position(&self) -> Pos {
        Pos {
            byte: self.byte.end,
            page: self.page.end,
        }
    }

    /// Returns the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.byte.len()
    }
}

impl<'text> PartialOrd for Span<'text> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.byte.partial_cmp(&other.byte)
    }
}

impl<'text> std::fmt::Display for Span<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({}, bytes {})", self.text(), self.page, self.byte)
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

    /// Generates the ByteSpan of the input text relative to the given byte.
    fn from_text_starting<'t>(start: usize, text: &'t str) -> Self {
        ByteSpan {
            start,
            end: start + text.len(),
        }
    }
}

impl Default for ByteSpan {
    fn default() -> Self {
        ByteSpan { start: 0, end: 0 }
    }
}

impl std::fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
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

    /// Generates the PageSpan of the input text relative to the given Page.
    fn from_text_starting<'t, N>(start: Page, text: &'t str, newline: N)
        -> Self
        where N: AsRef<str>,
    {
        PageSpan {
            start,
            end: start.advance(text, newline),
        }
    }
}

impl Default for PageSpan {
    fn default() -> Self {
        PageSpan { start: Page::ZERO, end: Page::ZERO }
    }
}

impl std::fmt::Display for PageSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Page
////////////////////////////////////////////////////////////////////////////////
/// A position with the source text identified by line and column numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Page {
    // NOTE: Field order must be maintained for PartialOrd and Ord impls.
    /// The line number.
    pub line: usize,
    /// The column number.
    pub column: usize,
}

impl Page {
    /// The start position.
    pub const ZERO: Page = Page { line: 0, column: 0 };

    /// Advances the Page by the contents of the given text.
    pub fn advance<S, N>(self, text: S, newline: N) -> Self
        where
            S: AsRef<str>,
            N: AsRef<str>,
    {
        let mut line = self.line;
        let mut column = self.column;
        let mut split = text.as_ref().split(newline.as_ref());


        match split.next() {
            Some(substr) if !substr.is_empty()
                // TODO: Avoid iterating over chars twice.
                => column += substr.chars().count(),

            _   => (),
        }

        for substr in split {
            line += 1;
            column = substr.chars().count();
        }

        Page { line, column }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page::ZERO
    }
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
