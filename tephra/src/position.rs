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
// Constants
////////////////////////////////////////////////////////////////////////////////

/// The default `LineEnding`.
pub const DEFAULT_LINE_ENDING: LineEnding = LineEnding::Lf;

/// The default tab width.
pub const DEFAULT_TAB_WIDTH: u8 = 4;

/// The byte size of a tab character.
const TAB_LEN_UTF8: usize = '\t'.len_utf8();


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
// LineEnding
////////////////////////////////////////////////////////////////////////////////
/// Line endings used to track page positioning in the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LineEnding {
    /// Lines end in a `'\n'` character.
    Lf,
    /// Lines end in a `'\r'` character.
    Cr,
    /// Lines end in a `'\r\n'` character sequence.
    CrLf,
}

impl Default for LineEnding {
    fn default() -> Self {
        DEFAULT_LINE_ENDING
    }
}

impl LineEnding {
    /// Returns the line ending as an `&'static str`.
    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf   => "\n",
            LineEnding::Cr   => "\r",
            LineEnding::CrLf => "\r\n",
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// ColumnMetrics
////////////////////////////////////////////////////////////////////////////////
/// Line ending and tab width measurements for column positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnMetrics {
    /// The source line ending.
    pub line_ending: LineEnding,
    /// The source tab width.
    pub tab_width: u8,
}

impl Default for ColumnMetrics {
    fn default() -> Self {
        ColumnMetrics::new()
    }
}

impl ColumnMetrics {
    /// Returns a new `ColumnMetrics` with the default values.
    pub const fn new() -> Self {
        ColumnMetrics {
            line_ending: DEFAULT_LINE_ENDING,
            tab_width: DEFAULT_TAB_WIDTH,
        }
    }

    /// Sets the line ending style for the Lexer.
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    /// Sets the tab width for the Lexer.
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.tab_width = tab_width;
        self
    }

    /// Returns the next column-aligned position after the given base position
    /// within the given text. None is returned if the result position is not
    /// within the text.
    pub fn next_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        let line_break = self.line_ending.as_str();

        if text[base.byte..].starts_with(line_break) {
            return Some(Pos::new(
                base.byte + line_break.len(),
                base.page.line + 1,
                0));
        }

        let mut chars = text[base.byte..].chars();
        match chars.next() {
            Some(c) if c == '\t' => {
                let tab = self.tab_width as usize;
                let tab_stop = tab - (base.page.column % tab);
                Some(Pos::new(
                    base.byte + TAB_LEN_UTF8,
                    base.page.line,
                    base.page.column + tab_stop))
            },

            Some(c) => Some(Pos::new(
                base.byte + c.len_utf8(),
                base.page.line,
                base.page.column + UnicodeWidthChar::width(c).unwrap_or(0))),

            None => None,
        }
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the given text. None is returned if the result position
    /// is not within the text.
    pub fn previous_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        let line_break = self.line_ending.as_str();

        if text[..base.byte].ends_with(line_break) {
            return Some(Pos::new(
                base.byte - line_break.len(),
                base.page.line - 1,
                0));
        }

        let mut chars = text[..base.byte].chars();
        match chars.next_back() {
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
                    if current.byte == base.byte - TAB_LEN_UTF8 { break }
                }
                
                Some(current)
            },

            Some(c) => Some(Pos::new(
                base.byte - c.len_utf8(),
                base.page.line,
                base.page.column - UnicodeWidthChar::width(c).unwrap_or(0))),

            None => None,
        }
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    pub fn is_line_break<'text>(&self, text: &'text str, byte: usize) -> bool {
        let line_break = self.line_ending.as_str();

        text[byte..].starts_with(line_break)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    pub fn line_end_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
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
    pub fn line_start_position<'text>(&self, text: &'text str, base: Pos) -> Pos {
        let line_break_len = self.line_ending.as_str().len();

        let mut start_byte = base.byte;
        while start_byte > 0 {
            while !text.is_char_boundary(start_byte) {
                start_byte -= 1;
            }
            if self.is_line_break(text, start_byte) {
                start_byte += line_break_len;
                break;
            }
            if start_byte > 0 { start_byte -= 1; }
        }

        Pos::new(start_byte, base.page.line, 0)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn previous_line_end_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        self.previous_position(
            text,
            self.line_start_position(text, base))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    pub fn next_line_start_position<'text>(&self, text: &'text str, base: Pos)
        -> Option<Pos>
    {
        self.next_position(
            text,
            self.line_end_position(text, base))
    }

    /// Returns the end position of the text, given its start position.
    pub fn end_position<'text>(&self, text: &'text str, start: Pos) -> Pos {
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
    pub fn start_position<'text>(&self, text: &'text str, end: Pos) -> Pos {
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
    pub fn position_after_str<'text, 'a>(
        &self,
        text: &'text str,
        start: Pos,
        pattern: &'a str)
        -> Option<Pos>
    {
        let mut end = start;
        while let Some(adv) = self.next_position(text, end) {
            if &pattern[end.byte-start.byte .. adv.byte-start.byte] 
                != &text[end.byte..adv.byte]
            {
                break;
            }
            if adv.byte - start.byte >= pattern.len() {
                return Some(adv);
            }
            end = adv;
        }
        None
    }

    /// Returns the position after any `char`s matching a closure, given its
    /// start position.
    pub fn position_after_chars_matching<'text, F>(
        &self,
        text: &'text str,
        start: Pos,
        mut f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        let mut end = start;
        while let Some(adv) = self.next_position(text, end) {
            if !text[end.byte..adv.byte].chars().all(&mut f) { break; }
            end = adv;
        }
        if end == start {
            None
        } else {
            Some(end)
        }
    }

    /// Returns the next position after `char`s matching a closure, given its
    /// start position.
    pub fn next_position_after_chars_matching<'text, F>(
        &self,
        text: &'text str,
        start: Pos,
        mut f: F)
        -> Option<Pos>
        where F: FnMut(char) -> bool
    {
        if let Some(adv) = self.next_position(text, start) {
            if text[start.byte..adv.byte].chars().all(&mut f) { 
                return Some(adv);
            }
        }
        None
    }

    /// Returns an iterator over the display columns of the given text.
    pub fn iter_columns<'text>(&self, text: &'text str, base: Pos)
        -> IterColumns<'text>
    {
        IterColumns {
            text,
            base: Some(base),
            metrics: *self,
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// IterColumns
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the display columns of a source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IterColumns<'text> {
    /// The source text.
    text: &'text str,
    /// The current column position.
    base: Option<Pos>,
    /// The column metrics.
    metrics: ColumnMetrics,
}

impl<'text> Iterator for IterColumns<'text> {
    type Item = (&'text str, Pos);

    fn next(&mut self) -> Option<Self::Item> {
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
