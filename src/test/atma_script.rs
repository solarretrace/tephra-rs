////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Scanner;
use crate::span::Pos;
use crate::span::NewLine;
use crate::span::Page;

// Standard library imports.
use std::convert::TryInto as _;




////////////////////////////////////////////////////////////////////////////////
// Scanner
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AtmaToken {
    CommandChunk,
    CommandTerminator,
    
    RawStringOpen,
    RawStringClose,
    RawStringText,
    
    StringOpenSingle,
    StringCloseSingle,
    StringOpenDouble,
    StringCloseDouble,
    StringText,

    LineCommentOpen,
    LineCommentText,
    
    Whitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtmaScriptScanner {
    open: Option<AtmaToken>,
    depth: u64,
}

impl AtmaScriptScanner {
    pub fn new() -> Self {
        AtmaScriptScanner {
            open: None,
            depth: 0,
        }
    }

    /// Parses a CommandChunk token.
    fn parse_command_chunk<Nl>(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
        where Nl: NewLine,
    {
        let tok = text
            .split(|c: char| c.is_whitespace() || c == ';')
            .next().unwrap();
        if tok.len() > 0 {
            let pos = Pos::new_from_string::<_, Nl>(tok);
            Some((AtmaToken::CommandChunk, pos))
        } else {
            None
        }
    }

    /// Parses a CommandTerminator token.
    fn parse_command_terminator(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with(";") {
            Some((AtmaToken::CommandTerminator, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a RawStringOpen token.
    fn parse_raw_string_open(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        match chars.next() {
            Some('r') => pos += Pos::new(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos += Pos::new(1, 0, 1);
                    self.depth += 1;
                    continue;
                },
                '"' => {
                    pos += Pos::new(1, 0, 1);
                    return Some((AtmaToken::RawStringOpen, pos));
                },
                _ => return None,
            }
        }

        None
    }

    /// Parses a RawStringClose token.
    fn parse_raw_string_close(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let mut raw_count = 0;
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        match chars.next() {
            Some('"') => pos += Pos::new(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos += Pos::new(1, 0, 1);
                    raw_count += 1;
                    if raw_count >= self.depth {
                        break;
                    }
                },
                _ => break,
            }
        }

        if self.depth == raw_count {
            Some((AtmaToken::RawStringClose, pos))
        } else {
            None
        }
    }

    /// Parses a RawStringText token.
    fn parse_raw_string_text<Nl>(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
        where Nl: NewLine,
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '"' {
                if self.parse_raw_string_close(&text[pos.byte..]).is_some() {
                    self.open = Some(AtmaToken::RawStringText);
                    return Some((AtmaToken::RawStringText, pos));
                }
            }

            if c == '\n' {
                pos += Pos::new(1, 1, 0);
            } else {
                pos += Pos::new(c.len_utf8(), 0, 1);
            }
        }

        None
    }

    /// Parses a StringOpenSingle token.
    fn parse_string_open_single(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringOpenSingle, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }
    
    /// Parses a StringCloseSingle token.
    fn parse_string_close_single(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringCloseSingle, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringOpenDouble token.
    fn parse_string_open_double(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringOpenDouble, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringCloseDouble token.
    fn parse_string_close_double(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringCloseDouble, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringText token.
    fn parse_string_text<Nl>(&mut self, text: &str, open: AtmaToken)
        -> Option<(AtmaToken, Pos)>
        where Nl: NewLine,
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        
        loop {
            // Skip past newline chars.
            if chars.as_str().starts_with(Nl::STR) {
                pos += Pos::new(Nl::len(), 1, 0);
                let _ = chars.nth(Nl::len() - 1);
                continue;
            }

            if let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        // NOTE: These should all step by column, because
                        // they're escaped text.
                        Some('\\') | 
                        Some('"')  | 
                        Some('\'') | 
                        Some('t')  | 
                        Some('r')  |
                        Some('n')  => pos += Pos::new(2, 0, 2),
                        Some('u')  => unimplemented!("unicode escapes unsupported"),
                        Some(_)    |
                        None       => return None,
                    }
                    continue;
                }

                match (c, open) {
                    ('\'', AtmaToken::StringOpenSingle) |
                    ('"', AtmaToken::StringOpenDouble)  => {
                        return Some((AtmaToken::StringText, pos));
                    },
                    _ => pos += Pos::new(c.len_utf8(), 0, 1),
                }
            } else {
                break;
            }
        }

        None
    }

    /// Parses a LineCommentOpen token.
    fn parse_line_comment_open(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("#") {
            Some((AtmaToken::LineCommentOpen, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a LineCommentText token.
    fn parse_line_comment_text<Nl>(&mut self, text: &str)
        -> (AtmaToken, Pos)
        where Nl: NewLine,
    {
        if let Some(first) = text.split(Nl::STR).next() {
            let span = Pos {
                byte: first.len(),
                page: Page { line: 0, column: first.len() },
            };
            (AtmaToken::LineCommentText, span)
        } else {
            unreachable!()
        }
    }

    /// Parses a Whitespace token.
    fn parse_whitespace<Nl>(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
        where Nl: NewLine,
    {
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let span = Pos::new_from_string::<_, Nl>(&text[0..substr_len]);
            Some((AtmaToken::Whitespace, span))
        } else {
            None
        }
    }
}

impl Scanner for AtmaScriptScanner {
    type Token = AtmaToken;

    fn scan<'text, Nl>(&mut self, text: &'text str)
        -> Option<(Self::Token, Pos)>
        where Nl: NewLine,
    {
        use AtmaToken::*;
        match self.open.take() {
            Some(LineCommentOpen) => {
                Some(self.parse_line_comment_text::<Nl>(text))
            },

            Some(RawStringText) => {
                // Because it is necessary to recognize the RawStringClose to
                // finish parsing RawStringText, we should never get here unless
                // we know the next part of the text is the appropriately sized
                // RawStringClose token. So instead of explicitely parsing it,
                // we can just jump forward.
                let byte: usize = (self.depth + 1)
                    .try_into()
                    .expect("Pos overflow");
                Some((RawStringClose, Pos::new(byte, 0, byte)))
            },
            Some(RawStringOpen) => {
                if let Some(parse) = self.parse_raw_string_close(text) {
                    self.depth = 0;
                    return Some(parse);
                }
                if let Some(parse) = self.parse_raw_string_text::<Nl>(text) {
                    return Some(parse);
                }
                None
            },

            Some(StringOpenSingle) => {
                if let Some(parse) = self.parse_string_close_single(text) {
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_string_text::<Nl>(text, StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                None
            },
            Some(StringOpenDouble) => {
                if let Some(parse) = self.parse_string_close_double(text) {
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_string_text::<Nl>(text, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }
                None
            },

            None => {
                if let Some(parse) = self.parse_command_terminator(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_whitespace::<Nl>(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_line_comment_open(text) {
                    self.open = Some(LineCommentOpen);
                    return Some(parse);
                }
                if let Some(parse) = self.parse_raw_string_open(text) {
                    self.open = Some(RawStringOpen);
                    return Some(parse);
                }
                if let Some(parse) = self.parse_string_open_single(text) {
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                if let Some(parse) = self.parse_string_open_double(text) {
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }
                if let Some(parse) = self.parse_command_chunk::<Nl>(text) {
                    return Some(parse);
                }

                None
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}
