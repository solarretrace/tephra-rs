////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Character positioning.
////////////////////////////////////////////////////////////////////////////////



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


    pub fn shifted(self, other: Self) -> Self {
        Pos {
            byte: self.byte + other.byte,
            page: self.page.shifted(other.page),
        }
    }

    pub fn shift(&mut self, other: Self) {
        self.byte += other.byte;
        self.page.shift(other.page);
    }

    pub(in crate) fn with_byte_offset<F>(mut self, offset: usize, f: F)
        -> Option<Self> 
        where F: FnOnce(Self) -> Option<Self>
    {
        self.byte -= offset;
        let res = (f)(self);
        res.map(|mut r| { r.byte += offset; r })
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

impl Default for Page {
    fn default() -> Self {
        Page::ZERO
    }
}

impl Page {
    /// The start position.
    pub const ZERO: Page = Page { line: 0, column: 0 };

    /// Return true if the page position is the start of a new line.
    pub fn is_line_start(self) -> bool {
        self.column == 0
    }

    pub fn shifted(self, other: Self) -> Self {
        Page {
            line: self.line + other.line,
            column: if other.is_line_start() || other.line > 0 { 
                other.column
            } else {
                self.column + other.column
            },
        }
    }

    pub fn shift(&mut self, other: Self) {
        self.line += other.line;
        if other.is_line_start() || other.line > 0 {
            self.column = other.column;
        } else {
            self.column += other.column;
        }
    }
}


impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

