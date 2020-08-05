////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer definition for Atma commands.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Local imports.
use crate::lexer::Scanner;
use crate::lexer::Lexer;
use crate::span::Pos;
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::ParseError;
use crate::result::Failure;
use crate::primitive::one;
use crate::combinator::both;
use crate::combinator::right;
use crate::primitive::any;
use crate::combinator::bracket_dynamic;
use crate::combinator::bracket;
use crate::combinator::exact;
use crate::combinator::text;

// Standard library imports.
use std::convert::TryInto as _;
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// AtmaToken
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AtmaToken {
    /// Any number of whitespace characters.
    Whitespace,
    
    /// An open parenthesis character '('.
    OpenParen,
    /// A close parenthesis character ')'.
    CloseParen,

    /// A raw string open "r[#*]\"".
    RawStringOpen,
    /// A raw string close "\"[#*]", which must match the corresponding open
    /// text.
    RawStringClose,
    /// A raw string, ignoring escape characters.
    RawStringText,
    
    /// A single-quote string open character '''.
    StringOpenSingle,
    /// A single-quote string close character '''.
    StringCloseSingle,
    /// A double-quote string open character '"'.
    StringOpenDouble,
    /// A double-quote string close character '"'.
    StringCloseDouble,
    /// A string with potential escape characters.
    StringText,

    /// A colon character ':'.
    Colon,
    /// A comma character ','.
    Comma,
    /// An octothorpe character '#'.
    Hash,
    /// An asterisk character '*'.
    Mult,
    /// A plus character '+'.
    Plus,
    /// A minus or hyphen character '-'.
    Minus,

    /// A floating point number.
    Float,
    /// A decimal point character '.'.
    Decimal,
    
    /// Any number of uint digits or underscore characters.
    Uint,
    /// An identifier with the form "[_[alpha]][alphanumeric]+".
    Ident,

    /// An underscore character '_'.
    Underscore,
    
}

////////////////////////////////////////////////////////////////////////////////
// AtmaScanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct AtmaScanner {
    open: Option<AtmaToken>,
    depth: u64,
}

impl AtmaScanner {
    pub fn new() -> Self {
        AtmaScanner {
            open: None,
            depth: 0,
        }
    }


    /// Parses a Ident token.
    fn parse_ident(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let mut bytes = 0;
        let mut cols = 0;
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if c.is_alphabetic() || c == '_' => {
                cols += 1;
                bytes += c.len_utf8();
            },
            _ => return None,
        }

        while let Some(c) = chars.next() {
            if c.is_alphanumeric() || c == '_' {
                cols += 1;
                bytes += c.len_utf8();
            } else {
                break;
            }
        }

        Some((AtmaToken::Ident, Pos::new(bytes, 0, cols)))
    }

    /// Parses an Uint token.
    fn parse_uint(&mut self, mut text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let radix = if text.starts_with("0b") {
            text = &text[2..];
            2
        } else if text.starts_with("0o") {
            text = &text[2..];
            8
        } else if text.starts_with("0x") {
            text = &text[2..];
            16
        } else {
            // Unprefixed uints can't start with '_'.
            if text.starts_with('_') { return None; }
            10
        };


        let rest = text
            .trim_start_matches(|c: char| c.is_digit(radix) || c == '_');
        
        let mut cols = text.len() - rest.len();
        if radix != 10 { cols += 2 }
        if cols > 0 {
            return Some((AtmaToken::Uint, Pos::new(cols, 0, cols)));
        } else {
            None
        }
    }

    /// Parses a Float token.
    fn parse_float(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("inf") {
            return Some((AtmaToken::Float, Pos::new(3, 0, 3)));
        }
        if text.starts_with("nan") {
            return Some((AtmaToken::Float, Pos::new(3, 0, 3)));
        }

        let mut rest = text.trim_start_matches(|c: char| c.is_digit(10));

        if rest.len() != text.len() {
            if rest.starts_with('.') {
                rest = &rest[1..];
            } else {
                return None;
            }

            let next = text.trim_start_matches(|c: char| c.is_digit(10));
            if next.len() != rest.len() {
                rest = next;

                // A single decimal is not a float.
                if rest.len() + 1 == text.len() { return None; }

                // Parse exponent or return.
                if rest.len() == 0 ||
                    !rest.starts_with(|c: char| c == 'e' || c == 'E')
                { 
                    let cols = text.len() - rest.len();
                    return Some((AtmaToken::Float, Pos::new(cols, 0, cols)));
                }
                rest = &rest[1..];

                if rest.starts_with('+') || rest.starts_with('-') { 
                    rest = &rest[1..];
                }
                if rest.len() == 0 { return None; }

                let next = text.trim_start_matches(|c: char| c.is_digit(10));
                if next.len() != rest.len() {
                    rest = next;
                    let cols = text.len() - rest.len();
                    return Some((AtmaToken::Float, Pos::new(cols, 0, cols)));
                }
            }
        }
        None
    }

    /// Parses a OpenParen token.
    fn parse_open_paren(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('(') {
            Some((AtmaToken::OpenParen, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }


    /// Parses a CloseParen token.
    fn parse_close_paren(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with(')') {
            Some((AtmaToken::CloseParen, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Decimal token.
    fn parse_decimal(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('.') {
            Some((AtmaToken::Decimal, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Colon token.
    fn parse_colon(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with(':') {
            Some((AtmaToken::Colon, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Comma token.
    fn parse_comma(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with(',') {
            Some((AtmaToken::Comma, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Hash token.
    fn parse_hash(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('#') {
            Some((AtmaToken::Hash, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Mult token.
    fn parse_mult(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('*') {
            Some((AtmaToken::Mult, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Plus token.
    fn parse_plus(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('+') {
            Some((AtmaToken::Plus, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a Minus token.
    fn parse_minus(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('-') {
            Some((AtmaToken::Minus, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }


    /// Parses a Underscore token.
    fn parse_underscore(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with('_') {
            Some((AtmaToken::Underscore, Pos::new(1, 0, 1)))
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

impl Scanner for AtmaScanner {
    type Token = AtmaToken;

    fn scan<'text, Nl>(&mut self, text: &'text str)
        -> Option<(Self::Token, Pos)>
        where Nl: NewLine,
    {
        use AtmaToken::*;
        match self.open.take() {
            
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
                if let Some(parse) = self.parse_whitespace::<Nl>(text) {
                    return Some(parse);
                }

                if let Some(parse) = self.parse_open_paren(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_close_paren(text) {
                    return Some(parse);
                }

                // RawStringOpen must be parsed before Hash.
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

                if let Some(parse) = self.parse_colon(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_comma(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_hash(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_mult(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_plus(text) {
                    return Some(parse);
                }
                if let Some(parse) = self.parse_minus(text) {
                    return Some(parse);
                }
                
                // Float must be parsed before Uint and Decimal.
                if let Some(parse) = self.parse_float(text) {
                    return Some(parse);
                }

                if let Some(parse) = self.parse_decimal(text) {
                    return Some(parse);
                }


                if let Some(parse) = self.parse_uint(text) {
                    return Some(parse);
                }
                // Ident must be parsed before Underscore.
                if let Some(parse) = self.parse_ident(text) {
                    return Some(parse);
                }

                if let Some(parse) = self.parse_underscore(text) {
                    return Some(parse);
                }
                
                None
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}
