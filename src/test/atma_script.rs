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
use crate::span::Page;

// Standard library imports.
use std::convert::TryInto as _;




////////////////////////////////////////////////////////////////////////////////
// Scanner.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenError(&'static str);
impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    newline_token: Box<str>,
    raw_string_bracket_count: u64,
}

impl AtmaScriptScanner {
    pub fn new() -> Self {
        AtmaScriptScanner {
            open: None,
            raw_string_bracket_count: 0,
            newline_token: "\n".into(),
        }
    }

    /// Parses a CommandChunk token.
    fn parse_command_chunk(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let tok = text
            .split(|c: char| c.is_whitespace() || c == ';')
            .next().unwrap();
        if tok.len() > 0 {
            let pos = Pos::new_from_string(tok, &*self.newline_token);
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
            Some('r') => pos.step(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos.step(1, 0, 1);
                    self.raw_string_bracket_count += 1;
                    continue;
                },
                '"' => {
                    pos.step(1, 0, 1);
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
            Some('"') => pos.step(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos.step(1, 0, 1);
                    raw_count += 1;
                    if raw_count >= self.raw_string_bracket_count {
                        break;
                    }
                },
                _ => break,
            }
        }

        if self.raw_string_bracket_count == raw_count {
            Some((AtmaToken::RawStringClose, pos))
        } else {
            None
        }
    }

    /// Parses a RawStringText token.
    fn parse_raw_string_text(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
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
                pos.step(1, 1, 0);
            } else {
                pos.step(c.len_utf8(), 0, 1);
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
    fn parse_string_text(&mut self, text: &str, open: AtmaToken)
        -> Option<(AtmaToken, Pos)>
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    // NOTE: These should all step by column, because they're
                    // escaped text.
                    Some('\\') | 
                    Some('"')  | 
                    Some('\'') | 
                    Some('t')  | 
                    Some('r')  |
                    Some('n')  => pos.step(2, 0, 2),
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
                ('\n', _) => pos.step(1, 1, 0),
                _         => pos.step(c.len_utf8(), 0, 1),
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
    fn parse_line_comment_text(&mut self, text: &str)
        -> (AtmaToken, Pos)
    {
        if let Some(first) = text.split(&*self.newline_token).next() {
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
    fn parse_whitespace(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let span = Pos::new_from_string(
                &text[0..substr_len],
                &*self.newline_token);
            Some((AtmaToken::Whitespace, span))
        } else {
            None
        }
    }
}

impl Scanner for AtmaScriptScanner {
    type Token = AtmaToken;
    type Error = TokenError;

    fn lex_prefix_token<'text>(&mut self, text: &'text str)
        -> Result<(Self::Token, Pos), (Self::Error, Pos)>
    {
        use AtmaToken::*;
        match self.open.take() {
            Some(LineCommentOpen) => {
                Ok(self.parse_line_comment_text(text))
            },

            Some(RawStringText) => {
                // Because it is necessary to recognize the RawStringClose to
                // finish parsing RawStringText, we should never get here unless
                // we know the next part of the text is the appropriately sized
                // RawStringClose token.
                let byte: usize = (self.raw_string_bracket_count + 1)
                    .try_into()
                    .expect("Pos overflow");
                Ok((RawStringClose, Pos::new(byte, 0, byte)))
            },
            Some(RawStringOpen) => {
                if let Some(parse) = self.parse_raw_string_close(text) {
                    self.raw_string_bracket_count = 0;
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_raw_string_text(text) {
                    Ok(parse)
                } else {
                    Err((TokenError("Non-terminated raw string"), Pos::ZERO))
                }
            },

            Some(StringOpenSingle) => {
                if let Some(parse) = self.parse_string_close_single(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text(text, StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    return Ok(parse);
                }
                Err((TokenError("Unrecognized string escape"),
                    Pos::new(1, 0, 1)))
            },
            Some(StringOpenDouble) => {
                if let Some(parse) = self.parse_string_close_double(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text(text, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Ok(parse);
                }
                Err((TokenError("Unrecognized string escape"),
                    Pos::new(1, 0, 1)))
            },

            None => {
                if let Some(parse) = self.parse_command_terminator(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_whitespace(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_line_comment_open(text) {
                    self.open = Some(LineCommentOpen);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_raw_string_open(text) {
                    self.open = Some(RawStringOpen);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_string_open_single(text) {
                    self.open = Some(StringOpenSingle);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_string_open_double(text) {
                    self.open = Some(StringOpenDouble);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_command_chunk(text) {
                    return Ok(parse);
                }

                Err((TokenError("Unrecognized token"), Pos::new(1, 0, 1)))
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}
