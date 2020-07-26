////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Text spans.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use few::Few;


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

impl<'text, Nl> From<Span<'text, Nl>> for OwnedSpan {
    fn from(other: Span<'text, Nl>) -> Self {
        OwnedSpan {
            text: other.source.to_owned().into(),
            byte: other.byte,
            page: other.page,
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Constructs the end position from the given string.
    pub fn new_from_string<S, Nl>(text: S) -> Self
        where
            S: AsRef<str>,
            Nl: NewLine,
    {
        let text = text.as_ref();
        Pos {
            byte: text.len(),
            page: Page::ZERO.advance::<_, Nl>(text),
        }
    }
}


impl std::ops::Add for Pos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Pos {
            byte: self.byte + other.byte,
            page: Page {
                line: self.page.line + other.page.line,
                column: if other.page.line == 0 { 
                    self.page.column + other.page.column
                } else {
                    other.page.column
                },
            },
        }
    }
}

impl std::ops::AddAssign for Pos {
    fn add_assign(&mut self, other: Self) {
        self.byte += other.byte;
        self.page.line += other.page.line;
        if other.page.line == 0 {
            self.page.column += other.page.column;
        } else {
            self.page.column = other.page.column;
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
// NOTE: Span methods must maintain an invariant: span.start() < span.end().
#[derive(Debug, Copy, Hash)]
pub struct Span<'text, Nl> {
    /// The newline type marker.
    newline: std::marker::PhantomData<Nl>,
    /// The source text.
    source: &'text str,
    /// The byte range of the spanned section within the source.
    byte: ByteSpan,
    /// The page range of the spanned section within the source.
    page: PageSpan,
}

impl<'text, Nl> Span<'text, Nl> {

    /// Constructs a new empty span in the given source text.
    pub fn new(source: &'text str) -> Self {
        Span {
            newline: std::marker::PhantomData::<Nl>,
            source,
            byte: ByteSpan::default(),
            page: PageSpan::default(),
        }
    }

    /// Constructs a new empty span in the given source text, starting from the
    /// given byte and page.
    pub fn new_from(pos: Pos, source: &'text str) -> Self {
        Span {
            newline: std::marker::PhantomData::<Nl>,
            source,
            byte: ByteSpan { start: pos.byte, end: pos.byte },
            page: PageSpan { start: pos.page, end: pos.page },
        }
    }

    /// Constructs a new span covering given source text.
    pub fn new_enclosing(a: Pos, b: Pos, source: &'text str) -> Self
    {
        let (a, b) = if a < b { (a, b) } else { (b, a) };

        let mut span = Self::new_from(a, source);
        span.extend_by(b);
        span
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

    /// Converts the span into and OwnedSpan.
    pub fn into_owned(self) -> OwnedSpan {
        OwnedSpan {
            text: self.source.to_owned().into(),
            byte: self.byte,
            page: self.page,
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

    /// Returns the span if it is non-empty, or None otherwise.
    pub fn into_nonempty(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
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

    /// Returns the smallest span covering the given spans.
    pub fn enclose(&self, other: &Self) -> Self {
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();
        let start = if a_start < b_start { a_start } else { b_start };
        let end = if a_end > b_end { a_end } else { b_end };
        Span::new_enclosing(start, end, self.source)
    }

    /// Returns the smallest set of spans covering the given spans.
    pub fn union(&self, other: &Self) -> Few<Self> {
        if self.intersects(other) {
            Few::One(self.enclose(other))
        } else {
            Few::Two(self.clone(), other.clone())
        }
    }

    /// Returns the overlapping portion the spans.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();

        let start = match (self.contains(&b_start), other.contains(&a_start)) {
            (true,  false) => b_start,
            (false, true)  => a_start,
            (false, false) => return None,
            (true,  true)  => unreachable!()
        };

        let end = match (self.contains(&b_end), other.contains(&a_end)) {
            (true,  false) => b_end,
            (false, true)  => a_end,
            (false, false) => return None,
            (true,  true)  => unreachable!()
        };

        Some(Span::new_enclosing(start, end, self.source))
    }

    /// Returns the result of removing a portion of the span.
    pub fn minus(&self, other: &Self) -> Few<Self> {
        let a_start = self.start();
        let b_start = other.start();
        let a_end = self.end();
        let b_end = other.end();

        let l = match (self.contains(&b_start), other.contains(&a_start)) {
            (true,  false) => Some((b_start, a_start)),
            (false, true)  => Some((a_start, b_start)),
            (false, false) => None,
            (true,  true)  => unreachable!(),
        };

        let r = match (self.contains(&b_end), other.contains(&a_end)) {
            (true,  false) => Some((b_end, a_end)),
            (false, true)  => Some((a_end, b_end)),
            (false, false) => None,
            (true,  true)  => unreachable!(),
        };

        let l = l.map(|(a, b)| Span::new_enclosing(a, b, self.source));
        let r = r.map(|(a, b)| Span::new_enclosing(a, b, self.source));
        Few::from((l, r))
    }
}

impl<'text, Nl> Span<'text, Nl> where Nl: NewLine {
    /// Constructs a new span covering given source text.
    pub fn full(source: &'text str) -> Self
    {
        let mut span = Self::new(source);
        span.extend_by_bytes(source.len());
        span
    }

    /// Extends the span by the given number of bytes.
    pub fn extend_by_bytes(&mut self, bytes: usize) {
        let substr = &self.source[self.byte.end..self.byte.end+bytes];

        self.byte = ByteSpan::from_text_starting(self.byte.end, substr);
        self.page = PageSpan::from_text_starting::<Nl>(self.page.start, substr);
    }


    /// Returns an iterator over the lines of the span.
    pub fn split_lines(&self) -> SplitLines<'text, Nl> {
        SplitLines {
            newline: self.newline,
            base: self.start(),
            text: self.text(),
            source: self.source,
            max_line: self.page.end.line,
        }
    }

    /// Widens the span on the left and right to the nearest newline.
    pub fn widen_to_line(&self) -> Self {
        if self.is_full() { return self.clone(); }
        
        let mut start_byte = self.byte.start;
        let mut end_byte = self.byte.end;
        let mut end_column = self.page.end.column;

        // Find the start byte.
        if self.page.start.line == 0 {
            start_byte = 0;
        } else {
            debug_assert!(start_byte > 0);
            let left = self.source[..start_byte]
                .rsplit_terminator(Nl::STR).next().unwrap();
            start_byte -= left.len();
        }
        // Find the end byte and column.
        let right = self.source[end_byte..]
            .split_terminator(Nl::STR).next().unwrap();        
        let right_pos = Pos::new_from_string::<_, Nl>(right);
        end_byte += right_pos.byte;
        debug_assert_eq!(right_pos.page.line, 0); // Should not cross any lines.
        end_column += right_pos.page.column; 

        Span {
            newline: std::marker::PhantomData::<Nl>,
            source: self.source,
            byte: ByteSpan {
                start: start_byte,
                end: end_byte,
            },
            page: PageSpan {
                start: Page { line: self.page.start.line, column: 0, },
                end: Page { line: self.page.end.line, column: end_column, },
            },
        }
    }

    /// Trims the span on the left and right, removing any whitespace.
    pub fn trim(&self) -> Self{
        let text = self.text();
        if text.is_empty() { return self.clone(); }
        
        let trimmed = text.trim_start();
        let left_len = text.len() - trimmed.len();
        let mut left_pos = self.start();
        left_pos += Pos::new_from_string::<_, Nl>(&text[..left_len]);
        let trimmed = trimmed.trim_end();
        let right_pos = Pos::new_from_string::<_, Nl>(trimmed);

        let mut span = Span::new_from(left_pos, self.source);
        span.extend_by(right_pos);

        span
    }
}

// Implement Clone manually to avoid requiring Nl: Clone.
impl<'text, Nl> Clone for Span<'text, Nl> {
    fn clone(&self) -> Self {
        Span {
            newline: std::marker::PhantomData::<Nl>,
            source: self.source,
            byte: self.byte,
            page: self.page,
        }
    }
}

// Implement PartialOrd manually to avoid requiring Nl: PartialOrd.
impl<'text, Nl> PartialOrd for Span<'text, Nl> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.byte.partial_cmp(&other.byte)
    }
}

// Implement Ord manually to avoid requiring Nl: Ord.
impl<'text, Nl> Ord for Span<'text, Nl> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
// Implement PartialEq manually to avoid requiring Nl: PartialEq.
impl<'text, Nl> PartialEq for Span<'text, Nl> {
    fn eq(&self, other: &Self) -> bool {
        self.byte == other.byte &&
        self.page == other.page &&
        self.source == other.source
    }
}

// Implement Eq manually to avoid requiring Nl: Eq.
impl<'text, Nl> Eq for Span<'text, Nl> {}


impl<'text, Nl> std::fmt::Display for Span<'text, Nl> {
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
    fn from_text_starting<'t, Nl>(start: Page, text: &'t str)
        -> Self
        where Nl: NewLine,
    {
        PageSpan {
            start,
            end: start.advance::<_, Nl>(text),
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
    pub fn advance<S, Nl>(self, text: S) -> Self
        where
            S: AsRef<str>,
            Nl: NewLine,
    {
        let mut line = self.line;
        let mut column = self.column;
        let mut split = text.as_ref().split(Nl::STR);


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




////////////////////////////////////////////////////////////////////////////////
// SplitLines
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the lines of a span. Returned by the `lines` method on
/// `Span`.
#[derive(Debug, Clone)]
pub struct SplitLines<'text, Nl> {
    newline: std::marker::PhantomData::<Nl>,
    base: Pos,
    text: &'text str,
    source: &'text str,
    max_line: usize,
}

impl<'text, Nl> Iterator for SplitLines<'text, Nl> where Nl: NewLine {
    type Item = Span<'text, Nl>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.base.page.line > self.max_line { return None; }
        if self.text.is_empty() {
            let mut span = Span::new_from(self.base, self.source);
            span.extend_by(Pos::new(0, 0, 0));
            self.base.page.line += 1;

            return Some(span);
        }

        if let Some(next) = self.text.split(Nl::STR).next() {
            
            let mut span = Span::new_from(self.base, self.source);

            self.base.page.line += 1;
            self.base.byte += Nl::len();
            
            let column = next.chars().count();
            self.base.page.column = column;
            self.base.byte += next.len();

            span.extend_by(Pos::new(next.len(), 0, column));

            self.text = &self.text[next.len() + Nl::len()..];
            self.base.page.column = 0;

            Some(span)
        } else {
            self.text = "";
            None
        }
    }
}

impl<'text, Nl> std::iter::FusedIterator for SplitLines<'text, Nl> 
    where Nl: NewLine,
{}




////////////////////////////////////////////////////////////////////////////////
// NewLine
////////////////////////////////////////////////////////////////////////////////
/// A trait representing the requirements for a Span's line separator.
pub trait NewLine: std::fmt::Debug + Clone + Copy + PartialEq + Eq 
    + PartialOrd + Ord + Default
{
    /// THe NewLine separator string.
    const STR: &'static str;

    /// Returns the byte length of the newline.
    fn len() -> usize {
        Self::STR.len()
    }
}

/// Carriage Return (`\r`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cr;

impl NewLine for Cr {
    const STR: &'static str = "\r";
}

/// Carriage Return - Line Feed (`\r\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CrLf;

impl NewLine for CrLf {
    const STR: &'static str = "\r\n";
}

/// Line Feed (`\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Lf;

impl NewLine for Lf {
    const STR: &'static str = "\n";
}
