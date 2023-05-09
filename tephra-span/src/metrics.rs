////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Column metrics for span generation.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Pos;

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
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf   => "\n",
            Self::Cr   => "\r",
            Self::CrLf => "\r\n",
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
        Self::new()
    }
}

impl ColumnMetrics {
    /// Returns a new `ColumnMetrics` with the default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line_ending: DEFAULT_LINE_ENDING,
            tab_width: DEFAULT_TAB_WIDTH,
        }
    }

    /// Sets the line ending style for the Lexer.
    #[must_use]
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    /// Sets the tab width for the Lexer.
    #[must_use]
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.tab_width = tab_width;
        self
    }

    /// Returns the next column-aligned position after the given base position
    /// within the given text. None is returned if the result position is not
    /// within the text.
    #[must_use]
    pub fn next_position(&self, text: &str, base: Pos)
        -> Option<Pos>
    {
        let line_break = self.line_ending.as_str();

        if text[base.byte..].starts_with(line_break) {
            let new_pos = Pos::new(
                base.byte + line_break.len(),
                base.page.line + 1,
                0);
            return Some(new_pos);
        }

        let mut chars = text[base.byte..].chars();
        match chars.next() {
            Some(c) if c == '\t' => {
                let tab = self.tab_width as usize;
                let tab_stop = tab - (base.page.column % tab);
                let new_pos = Pos::new(
                    base.byte + TAB_LEN_UTF8,
                    base.page.line,
                    base.page.column + tab_stop);

                Some(new_pos)
            },

            Some(c) => {
                let new_pos = Pos::new(
                    base.byte + c.len_utf8(),
                    base.page.line,
                    base.page.column + UnicodeWidthChar::width(c).unwrap_or(0));
                Some(new_pos)
            },

            None => None,
        }
    }

    /// Returns the previous column-aligned position before the given base
    /// position within the given text. None is returned if the result position
    /// is not within the text.
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn previous_position(&self, text: &str, base: Pos)
        -> Option<Pos>
    {
        let line_break = self.line_ending.as_str();

        if text[..base.byte].ends_with(line_break) {
            let new_pos = Pos::new(
                base.byte - line_break.len(),
                base.page.line - 1,
                0);
            return Some(new_pos);
        }

        let mut chars = text[..base.byte].chars();
        match chars.next_back() {
            Some(c) if c == '\t' => {
                // To get position of tab start, we must measure from the start
                // of the line.

                // Advance until we find the tab just before the current
                // position.
                let mut new_pos = self.line_start_position(text, base);
                loop {
                    let next = self.next_position(text, new_pos)
                        .expect("next position is guaranteed");
                    new_pos = next;
                    if new_pos.byte == base.byte - TAB_LEN_UTF8 { break }
                }

                
                Some(new_pos)
            },

            Some(c) => {
                let new_pos = Pos::new(
                    base.byte - c.len_utf8(),
                    base.page.line,
                    base.page.column - UnicodeWidthChar::width(c).unwrap_or(0));
                Some(new_pos)
            },

            None => None,
        }
    }

    /// Returns true if a line break is positioned at the given byte position in
    /// the text.
    #[must_use]
    pub fn is_line_break(&self, text: &str, byte: usize) -> bool {
        let line_break = self.line_ending.as_str();

        text[byte..].starts_with(line_break)
    }

    /// Returns the position of the end of line containing the given base
    /// position.
    #[must_use]
    pub fn line_end_position(&self, text: &str, base: Pos) -> Pos {
        let mut end = base;
        while end.byte < text.len() {
            if self.is_line_break(text, end.byte) {
                break;
            }
            match self.next_position(text, end) {
                Some(new) => end = new,
                None      => break,
            }
        }
        end
    }

    /// Returns the position of the start of line containing the given base
    /// position.
    #[must_use]
    pub fn line_start_position(&self, text: &str, base: Pos)
        -> Pos
    {
        let mut start_byte = base.byte;
        while start_byte > 0 {
            while !text.is_char_boundary(start_byte - 1) {
                start_byte -= 1;
            }
            if self.is_line_break(text, start_byte - 1) {
                break;
            }
            start_byte = start_byte.saturating_sub(1);
        }

        
        Pos::new(start_byte, base.page.line, 0)
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    #[must_use]
    pub fn previous_line_end_position(&self, text: &str, base: Pos)
        -> Option<Pos>
    {
        self.previous_position(
            text,
            self.line_start_position(text, base))
    }

    /// Returns the position at the start of the next line after the given base
    /// position.
    #[must_use]
    pub fn next_line_start_position(&self, text: &str, base: Pos)
        -> Option<Pos>
    {
        self.next_position(
            text,
            self.line_end_position(text, base))
    }

    /// Returns the start position of the text, given its end position.
    #[must_use]
    pub fn start_position(&self, text: &str, end: Pos) -> Pos {
        let mut start = end;

        while start.byte > 0 {
            match self.previous_position(text, start) {
                Some(new) => start = new,
                None      => break,
            }
        }
        start
    }

    /// Returns the end position of the text, given its start position.
    #[must_use]
    pub fn end_position(&self, text: &str, start: Pos) -> Pos {
        let mut end = start;

        while end.byte < text.len() {
            match self.next_position(text, end) {
                Some(new) => end = new,
                None      => break,
            }
        }
        end
    }

    /// Returns the position after the given pattern string, given its start
    /// position.
    #[must_use]
    pub fn position_after_str(
        &self,
        text: &str,
        start: Pos,
        pattern: &str)
        -> Option<Pos>
    {
        let mut end = start;
        while let Some(adv) = self.next_position(text, end) {
            if pattern[end.byte-start.byte .. adv.byte-start.byte] 
                != text[end.byte..adv.byte]
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
    pub fn position_after_chars_matching<F>(
        &self,
        text: &str,
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
    pub fn next_position_after_chars_matching<F>(
        &self,
        text: &str,
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
}
