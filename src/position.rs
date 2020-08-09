////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Character positioning.
////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////
// ColumnMetrics
////////////////////////////////////////////////////////////////////////////////
/// A trait representing the requirements for a Span's line separator.
pub trait ColumnMetrics: std::fmt::Debug + Clone + Copy + PartialEq + Eq 
    + PartialOrd + Ord + Default
{
    /// THe ColumnMetrics separator string.
    const STR: &'static str;

    /// Returns the byte length of the newline.
    fn len() -> usize {
        Self::STR.len()
    }
}

/// Carriage Return (`\r`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cr {
    tab_width: u8,
}

impl Cr {
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        Cr { tab_width }
    }
}

impl ColumnMetrics for Cr {
    const STR: &'static str = "\r";
}

/// Carriage Return - Line Feed (`\r\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CrLf {
    tab_width: u8,
}

impl CrLf {
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        CrLf { tab_width }
    }
}

impl ColumnMetrics for CrLf {
    const STR: &'static str = "\r\n";
}

/// Line Feed (`\n`) newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Lf {
    tab_width: u8,
}

impl Lf {
    pub fn with_tab_width(tab_width: u8) -> Self {
        assert!(tab_width > 0);
        Lf { tab_width }
    }
}

impl ColumnMetrics for Lf {
    const STR: &'static str = "\n";
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
    pub fn new_from_string<S, Cm>(text: S) -> Self
        where
            S: AsRef<str>,
            Cm: ColumnMetrics,
    {
        let text = text.as_ref();
        Pos {
            byte: text.len(),
            page: Page::ZERO.advance::<Cm>(text.as_ref()),
        }
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

    /// Advances the Page by the contents of the given text.
    pub fn advance<'t, Cm>(mut self, text: &'t str) -> Self
        where Cm: ColumnMetrics,
    {
        let mut chars = text.chars();
        loop {
            // Skip past newline chars.
            if chars.as_str().starts_with(Cm::STR) {
                self.line += 1;
                self.column = 0;
                let _ = chars.nth(Cm::len() - 1);
                continue;
            }

            if chars.next().is_none() { break; }
            self.column += 1;
        }

        self
    }
}

impl std::ops::Add for Page {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Page {
            line: self.line + other.line,
            column: if other.line == 0 { 
                self.column + other.column
            } else {
                other.column
            },
        }
    }
}

impl std::ops::AddAssign for Page {
    fn add_assign(&mut self, other: Self) {
        self.line += other.line;
        if other.line == 0 {
            self.column += other.column;
        } else {
            self.column = other.column;
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
