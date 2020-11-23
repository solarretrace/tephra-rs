////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Character positioning.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
// TODO: Use the unicode-segmentation crate instead?
use unicode_width::UnicodeWidthChar;


////////////////////////////////////////////////////////////////////////////////
// ColumnMetrics
////////////////////////////////////////////////////////////////////////////////
/// A trait providing column positioning measurements.
pub trait ColumnMetrics: std::fmt::Debug + Clone + Copy {
    /// Returns the next column-aligned position after the given base position
    /// within the given text. None is returned if the result position is not
    /// within the text.
    fn next_position<'text>(&self, text: &'text str, base: Pos) -> Option<Pos>;

    /// Returns the previous column-aligned position before the given base
    /// position within the given text. None is returned if the result position
    /// is not within the text.
    fn previous_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>;

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    fn is_line_break<'text>(&self, text: &'text str, byte: usize) -> bool;

    /// Returns the position of the end of line containing the given base
    /// position.
    fn line_end_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let mut end = base;
        while end.byte < text.len() {
            if self.is_line_break(text, end.byte) { break; }
            match self.next_position(text, end) {
                Some(new) => end = new,
                None      => break,
            }
        }
        end
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    fn line_start_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let mut start = base;

        while start.byte > 0 {
            match self.previous_position(text, start) {
                Some(new) => {
                    if new.page.line != base.page.line { break; }
                    start = new;
                },
                None      => break,
            }   
        }
        start
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    fn previous_line_end_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        self.previous_position(
            text,
            self.line_start_position(text, base))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    fn next_line_start_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        self.next_position(
            text,
            self.line_end_position(text, base))
    }

    /// Returns the end position of the text, given its start position.
    fn end_position<'text>(&self, text: &'text str, start: Pos) -> Pos {
        let mut end = start;

        while end.byte < text.len() {
            match self.next_position(text, end) {
                Some(new) => end = new,
                None      => break,
            }
        }
        end
    }

    /// Returns the start position of the text, given its end position.
    fn start_position<'text>(&self, text: &'text str, end: Pos) -> Pos {
        let mut start = end;

        while start.byte > 0 {
            match self.previous_position(text, start) {
                Some(new) => start = new,
                None      => break,
            }
        }
        start
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    fn position_after_str<'text, 'a>(
        &self,
        text: &'text str,
        start: Pos,
        pattern: &'a str)
        -> Pos
    {
        let mut end = start;
        while let Some(adv) = self.next_position(text, end) {
            if &pattern[end.byte-start.byte .. adv.byte-start.byte] 
                != &text[end.byte..adv.byte]
            {
                break;
            }
            end = adv;
            if adv.byte - start.byte >= pattern.len() { break; }
        }
        return end;
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    fn position_after_chars_matching<'text, F>(
        &self,
        text: &'text str,
        start: Pos,
        mut f: F)
        -> Pos
        where F: FnMut(char) -> bool
    {
        let mut end = start;
        while let Some(adv) = self.next_position(text, end) {
            if !text[end.byte..adv.byte].chars().all(&mut f) { break; }
            end = adv;
        }
        end
    }

    /// Returns an iterator over the display columns of the given text.
    fn iter_columns<'text>(&self, text: &'text str, base: Pos)
        -> IterColumns<'text, Self>
    {
        IterColumns {
            text,
            current: Some(base),
            metrics: *self,
        }
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

/// The byte-width of a line break for Lf.
const LF_LINE_BREAK_LEN: usize = 1;

/// The byte-width of a tab for Lf.
const LF_TAB_LEN: usize = 1;

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
    fn next_position<'text>(&self, text: &'text str, base: Pos) -> Option<Pos> {
        let mut chars = text[base.byte..].chars();
        
        match chars.next() {
            Some(c) if c == '\n' => Some(Pos::new(
                base.byte + LF_LINE_BREAK_LEN,
                base.page.line + 1,
                0)),
            
            Some(c) if c == '\t' => {
                let tab = self.tab_width as usize;
                let tab_stop = tab - (base.page.column % tab);
                Some(Pos::new(
                    base.byte + LF_TAB_LEN,
                    base.page.line,
                    base.page.column + tab_stop))
            },

            Some(c)              => Some(Pos::new(
                base.byte + c.len_utf8(),
                base.page.line,
                base.page.column + UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }

    fn previous_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        let mut chars = text[..base.byte].chars();
        match chars.next_back() {
            Some(c) if c == '\n' => Some(Pos::new(
                base.byte - LF_LINE_BREAK_LEN,
                base.page.line - 1,
                0)),
            
            Some(c) if c == '\t' => {
                // To get position of tab start, we must measure from the start
                // of the line.

                // Advance until we find the tab just before the current
                // position.
                let mut current = self.line_start_position(text, base);
                loop {
                    let next = self.next_position(text, current)
                        .expect("next position is guaranteed");
                    current = next;
                    if current.byte == base.byte - LF_TAB_LEN { break }
                }
                
                Some(current)
            },

            Some(c)              => Some(Pos::new(
                base.byte - c.len_utf8(),
                base.page.line,
                base.page.column - UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }


    fn line_start_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let mut start_byte = base.byte;
        while start_byte > 0 {
            while !text.is_char_boundary(start_byte) {
                start_byte -= 1;
            }
            if self.is_line_break(text, start_byte) {
                start_byte += LF_LINE_BREAK_LEN;
                break;
            }
            if start_byte > 0 { start_byte -= 1; }
        }

        Pos::new(start_byte, base.page.line, 0)
    }

    fn is_line_break<'text>(&self, text: &'text str, byte: usize) -> bool {
        &text[byte..byte + LF_LINE_BREAK_LEN] == "\n"
    }
}


impl Default for Lf {
    fn default() -> Self {
        Lf::new()
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

/// The byte-width of a line break for CrLf.
const CRLF_LINE_BREAK_LEN: usize = 2;

/// The byte-width of a tab for CrLf.
const CRLF_TAB_LEN: usize = 1;

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
fn next_position<'text>(&self, text: &'text str, base: Pos) -> Option<Pos> {
        let mut chars = text[base.byte..].chars();
        
        match chars.next() {
            Some(c) if c == '\r' => match chars.next() {
                Some(c) if c == '\n' => Some(Pos::new(
                    base.byte + CRLF_LINE_BREAK_LEN,
                    base.page.line + 1,
                    0)),
                _                    => Some(Pos::new(1, 0, 1)),
            }
            
            Some(c) if c == '\t' => {
                let tab = self.tab_width as usize;
                let tab_stop = tab - (base.page.column % tab);
                Some(Pos::new(
                    base.byte + CRLF_TAB_LEN,
                    base.page.line,
                    base.page.column + tab_stop))
            },

            Some(c)              => Some(Pos::new(
                base.byte + c.len_utf8(),
                base.page.line,
                base.page.column + UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }

    fn previous_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        let mut chars = text[..base.byte].chars();
        match chars.next_back() {
            Some(c) if c == '\n' => Some(Pos::new(
                base.byte - CRLF_LINE_BREAK_LEN,
                base.page.line - 1,
                0)),
            
            Some(c) if c == '\t' => {
                // To get position of tab start, we must measure from the start
                // of the line.

                // Advance until we find the tab just before the current
                // position.
                let mut current = self.line_start_position(text, base);
                loop {
                    let next = self.next_position(text, current)
                        .expect("next position is guaranteed");
                    current = next;
                    if current.byte == base.byte - CRLF_TAB_LEN { break }
                }
                
                Some(current)
            },

            Some(c)              => Some(Pos::new(
                base.byte - c.len_utf8(),
                base.page.line,
                base.page.column - UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }

    fn line_start_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let mut start_byte = base.byte;
        while start_byte > 0 {
            while !text.is_char_boundary(start_byte) {
                start_byte -= 1;
            }
            if self.is_line_break(text, start_byte) {
                start_byte += CRLF_LINE_BREAK_LEN;
                break;
            }
            if start_byte > 0 { start_byte -= 1; }
        }

        Pos::new(start_byte, base.page.line, 0)
    }

    fn is_line_break<'text>(&self, text: &'text str, byte: usize) -> bool {
        &text[byte..byte + CRLF_LINE_BREAK_LEN] == "\r\n"
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

/// The byte-width of a line break for Cr.
const CR_LINE_BREAK_LEN: usize = 1;

/// The byte-width of a tab for Cr.
const CR_TAB_LEN: usize = 1;

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
    fn next_position<'text>(&self, text: &'text str, base: Pos) -> Option<Pos> {
        let mut chars = text[base.byte..].chars();
        
        match chars.next() {
            Some(c) if c == '\n' => Some(Pos::new(
                base.byte + CR_LINE_BREAK_LEN,
                base.page.line + 1,
                0)),
            
            Some(c) if c == '\t' => {
                let tab = self.tab_width as usize;
                let tab_stop = tab - (base.page.column % tab);
                Some(Pos::new(
                    base.byte + CR_TAB_LEN,
                    base.page.line,
                    base.page.column + tab_stop))
            },

            Some(c)              => Some(Pos::new(
                base.byte + c.len_utf8(),
                base.page.line,
                base.page.column + UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }

    fn previous_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        let mut chars = text[..base.byte].chars();
        match chars.next_back() {
            Some(c) if c == '\n' => Some(Pos::new(
                base.byte - CR_LINE_BREAK_LEN,
                base.page.line - 1,
                0)),
            
            Some(c) if c == '\t' => {
                // To get position of tab start, we must measure from the start
                // of the line.

                // Advance until we find the tab just before the current
                // position.
                let mut current = self.line_start_position(text, base);
                loop {
                    let next = self.next_position(text, current)
                        .expect("next position is guaranteed");
                    current = next;
                    if current.byte == base.byte - CR_TAB_LEN { break }
                }
                
                Some(current)
            },

            Some(c)              => Some(Pos::new(
                base.byte - c.len_utf8(),
                base.page.line,
                base.page.column - UnicodeWidthChar::width(c).unwrap_or(0))),

            None                 => None,
        }
    }

    fn line_start_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let mut start_byte = base.byte;
        while start_byte > 0 {
            while !text.is_char_boundary(start_byte) {
                start_byte -= 1;
            }
            if self.is_line_break(text, start_byte) {
                start_byte += CR_LINE_BREAK_LEN;
                break;
            }
            if start_byte > 0 { start_byte -= 1; }
        }

        Pos::new(start_byte, base.page.line, 0)
    }

    fn is_line_break<'text>(&self, text: &'text str, byte: usize) -> bool {
        &text[byte..byte + CR_LINE_BREAK_LEN] == "\n"
    }
}

impl Default for Cr {
    fn default() -> Self {
        Cr::new()
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
    /// The current column position.
    current: Option<Pos>,
    /// The column metrics.
    metrics: Cm,
}

impl<'text, Cm> Iterator for IterColumns<'text, Cm>
    where Cm: ColumnMetrics,
{
    type Item = (&'text str, Pos);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current
            .and_then(|curr| self.metrics.next_position(self.text, curr));
        self.current = next;
        
        next.map(|p| (&self.text[p.byte..], p))
    }
}

impl<'text, Cm> std::iter::FusedIterator for IterColumns<'text, Cm>
    where Cm: ColumnMetrics,
{}
