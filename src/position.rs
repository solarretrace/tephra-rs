////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Character positioning.
////////////////////////////////////////////////////////////////////////////////

use unicode_width::UnicodeWidthChar;

////////////////////////////////////////////////////////////////////////////////
// ColumnMetrics
////////////////////////////////////////////////////////////////////////////////
/// A trait representing the requirements for a Span's line separator.
pub trait ColumnMetrics: std::fmt::Debug + Clone + Copy {

    /// Returns the position of the next display column after the start of the
    /// given text.
    fn next_column<'text>(&self, text: &'text str) -> Option<Pos>;

    /// Returns the position of the next display column before the end of the
    /// given text.
    fn next_back_column<'text>(&self, text: &'text str) -> Option<Pos>;

    /// Returns the display width of the given text.
    fn width<'text>(&self, text: &'text str) -> Pos {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            end += self.next_column(rest)
                .expect("column position for nonempty text");
            rest = &text[end.byte..];
        }
        end
    }

    /// Returns the position of the start of the next line after the start of
    /// the given text. If the text ends before a new line is encountered, the
    /// full width is returned.
    fn next_line_start<'text>(&self, text: &'text str) -> Pos {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            end += self.next_column(rest)
                .expect("column position for nonempty text");

            if end.page.column == 0 { break; }
            rest = &text[end.byte..];
        }
        end
    }

    /// Returns the position of the end of the current line after the start of
    /// the given text. If the text ends before a new line is encountered, the
    /// full width is returned.
    fn next_line_end<'text>(&self, text: &'text str) -> Pos {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            let next = self.next_column(rest)
                .expect("column position for nonempty text");

            if next.page.column == 0 { break; }
            end += next;
            rest = &text[end.byte..];
        }
        end
    }

    /// Returns the position up to first display column for which the given
    /// predicate fails from the start of the given text.
    fn next_until<'text, F>(&self, text: &'text str, mut pred: F) -> Pos 
        where F: for<'a> FnMut(&'a str, Pos) -> bool,
    {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            let next = self.next_column(rest)
                .expect("column position for nonempty text");

            if pred(&text[end.byte .. end.byte + next.byte], next) { break; }

            end += next;
            rest = &text[end.byte..];
        }
        end
    }

    /// Returns the position of the end of the next to last line of the given
    /// text. If the text ends before a new line is encountered, the full width
    /// is returned.
    fn next_back_line_start<'text>(&self, text: &'text str) -> Pos {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            let next = self.next_back_column(rest)
                .expect("column position for nonempty text");

            if next.page.column == 0 { break; }
            end += next;
            rest = &text[0..(text.len() - end.byte)];
        }
        end
    }

    /// Returns the position of the start of the last line of the given text.
    /// If the text ends before a new line is encountered, the full width is
    /// returned.
    fn next_back_line_end<'text>(&self, text: &'text str) -> Pos {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            end += dbg!(self.next_back_column(rest))
                .expect("column position for nonempty text");

            if end.page.column == 0 { break; }
            rest = &text[0..(text.len() - end.byte)];
        }
        end
    }

    /// Returns the position up to first display column for which the given
    /// predicate fails from the end of the given text.
    fn next_back_until<'text, F>(&self, text: &'text str, mut pred: F) -> Pos 
        where F: for<'a> FnMut(&'a str, Pos) -> bool,
    {
        let mut end = Pos::ZERO;
        let mut rest = text;

        while !rest.is_empty() {
            let next = self.next_back_column(rest)
                .expect("column position for nonempty text");

            let start = text.len() - end.byte;
            if pred(&text[start .. start + next.byte], next) { break; }

            end += next;
            rest = &text[end.byte..];
        }
        end
    }

    /// Returns an iterator over the display columns of the given text.
    fn iter_columns<'text>(&self, text: &'text str) -> IterColumns<'text, Self>
    {
        IterColumns {
            text,
            metrics: *self,
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// CrLf
////////////////////////////////////////////////////////////////////////////////
/// Carriage Return - Line Feed (`\r\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrLf {
    tab_width: u8,
}

impl CrLf {
    /// Returns a new `CrLf` with the default tab width of 4.
    pub fn new() -> Self {
        CrLf {
            tab_width: 4,
        }
    }

    /// Returns a new `CrLf` with the given tab width.
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        CrLf { tab_width }
    }
}

impl ColumnMetrics for CrLf {

    fn next_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if c == '\r' => match chars.next() {
                Some(c) if c == '\n' => Some(Pos::new(2, 1, 0)),
                _                    => Some(Pos::new(1, 0, 1)),
            }
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }

    fn next_back_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next_back() {
            Some(c) if c == '\n' => match chars.next_back() {
                Some(c) if c == '\r' => Some(Pos::new(2, 1, 0)),
                _                    => Some(Pos::new(1, 0, 1)),
            }
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }
}

impl Default for CrLf {
    fn default() -> Self {
        CrLf::new()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Cr
////////////////////////////////////////////////////////////////////////////////
/// Carriage Return (`\r`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cr {
    tab_width: u8,
}

impl Cr {
    /// Returns a new `Cr` with the default tab width of 4.
    pub fn new() -> Self {
        Cr {
            tab_width: 4,
        }
    }

    /// Returns a new `Cr` with the given tab width.
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        Cr { tab_width }
    }
}

impl ColumnMetrics for Cr {

    fn next_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if c == '\r' => Some(Pos::new(1, 1, 0)),
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }
    
    fn next_back_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next_back() {
            Some(c) if c == '\r' => Some(Pos::new(1, 1, 0)),
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }
}

impl Default for Cr {
    fn default() -> Self {
        Cr::new()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Lf
////////////////////////////////////////////////////////////////////////////////
/// Line Feed (`\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lf {
    tab_width: u8,
}

impl Lf {
    /// Returns a new `Lf` with the default tab width of 4.
    pub fn new() -> Self {
        Lf {
            tab_width: 4,
        }
    }

    /// Returns a new `Lf` with the given tab width.
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        Lf { tab_width }
    }
}

impl ColumnMetrics for Lf {
    fn next_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if c == '\n' => Some(Pos::new(1, 1, 0)),
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }

    fn next_back_column<'text>(&self, text: &'text str) -> Option<Pos> {
        let mut chars = text.chars();
        match chars.next_back() {
            Some(c) if c == '\n' => Some(Pos::new(1, 1, 0)),
            Some(c) if c == '\t' => Some(Pos::new(1, 0, self.tab_width as usize)),
            Some(c)              => Some(Pos::new(
                c.len_utf8(),
                0,
                UnicodeWidthChar::width(c).unwrap_or(0))),
            None                 => None,
        }
    }
}


impl Default for Lf {
    fn default() -> Self {
        Lf::new()
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

    /// Return true if the span position is the start of a new line.
    pub fn is_line_start(self) -> bool {
        self.page.is_line_start()
    }
}


impl std::ops::Add for Pos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Pos {
            byte: self.byte + other.byte,
            page: self.page + other.page,
        }
    }
}

impl std::ops::AddAssign for Pos {
    fn add_assign(&mut self, other: Self) {
        self.byte += other.byte;
        self.page += other.page;
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
// Page
////////////////////////////////////////////////////////////////////////////////
/// A position with the source text identified by line and column numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Page {
    // NOTE: Field order must be (line, col) for PartialOrd and Ord impls.
    /// The line number.
    pub line: usize,
    /// The column number.
    pub column: usize,
}

impl Page {
    /// The start position.
    pub const ZERO: Page = Page { line: 0, column: 0 };

    /// Return true if the page position is the start of a new line.
    pub fn is_line_start(self) -> bool {
        self.column == 0
    }
}

impl std::ops::Add for Page {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Page {
            line: self.line + other.line,
            column: if other.is_line_start() || other.line > 0 { 
                other.column
            } else {
                self.column + other.column
            },
        }
    }
}

impl std::ops::AddAssign for Page {
    fn add_assign(&mut self, other: Self) {
        self.line += other.line;
        if other.is_line_start() || other.line > 0 {
            self.column = other.column;
        } else {
            self.column += other.column;
        }
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
// IterColumns
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the display columns of a source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IterColumns<'text, Cm> {
    /// The source text.
    text: &'text str,
    /// The column metrics.
    metrics: Cm,
}

impl<'text, Cm> Iterator for IterColumns<'text, Cm>
    where Cm: ColumnMetrics,
{
    type Item = (&'text str, Pos);

    fn next(&mut self) -> Option<Self::Item> {
        self.metrics
            .next_column(self.text)
            .map(|pos| {
                let res = (&self.text[..pos.byte], pos);
                self.text = &self.text[pos.byte..];
                res
            })
    }
}

impl<'text, Cm> DoubleEndedIterator for IterColumns<'text, Cm>
    where Cm: ColumnMetrics,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.metrics
            .next_back_column(self.text)
            .map(|pos| {
                let res = (&self.text[pos.byte..], pos);
                self.text = &self.text[..(self.text.len() - pos.byte)];
                res
            })
    }
}

impl<'text, Cm> std::iter::FusedIterator for IterColumns<'text, Cm>
    where Cm: ColumnMetrics,
{}
