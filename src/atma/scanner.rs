////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Scanner definition for Atma commands.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Local imports.
use crate::lexer::Scanner;
use crate::lexer::Lexer;
use crate::position::Pos;
use crate::position::ColumnMetrics;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::ParseError;
use crate::result::Failure;
use crate::combinator::one;
use crate::combinator::both;
use crate::combinator::right;
use crate::combinator::any;
use crate::combinator::bracket_dynamic;
use crate::combinator::bracket;
use crate::combinator::exact;
use crate::combinator::text;

// Standard library imports.
use std::convert::TryInto as _;
use std::borrow::Cow;


macro_rules! return_if_some {
    ($p:expr) => {
        if let Some(parse) = $p {
            return Some(parse);
        }
    }
}


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

    /// An open square bracket character '['.
    OpenBracket,
    /// A close square bracket character ']'.
    CloseBracket,

    /// An open curly bracket character '{'.
    OpenBrace,
    /// A close curly bracket character '}'.
    CloseBrace,

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
    /// Any number of hex digits. Can only be parsed imediately following a
    /// Hash token.
    HexDigits,
    /// An identifier with the form "[_[alpha]][alphanumeric]+".
    Ident,

    /// An underscore character '_'.
    Underscore,
}

impl std::fmt::Display for AtmaToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AtmaToken::*;
        match self {
            Whitespace        => write!(f, "whitespace"),
            OpenParen         => write!(f, "'('"),
            CloseParen        => write!(f, "')'"),
            OpenBracket       => write!(f, "'['"),
            CloseBracket      => write!(f, "']'"),
            OpenBrace         => write!(f, "'{{'"),
            CloseBrace        => write!(f, "'}}'"),
            RawStringOpen     => write!(f, "raw string quote"),
            RawStringClose    => write!(f, "raw string quote"),
            RawStringText     => write!(f, "raw string text"),
            StringOpenSingle  => write!(f, "'''"),
            StringCloseSingle => write!(f, "'''"),
            StringOpenDouble  => write!(f, "'\"'"),
            StringCloseDouble => write!(f, "'\"'"),
            StringText        => write!(f, "string text"),
            Colon             => write!(f, "':'"),
            Comma             => write!(f, "','"),
            Hash              => write!(f, "'#'"),
            Mult              => write!(f, "'*'"),
            Plus              => write!(f, "'+'"),
            Minus             => write!(f, "'-'"),
            Float             => write!(f, "float"),
            Decimal           => write!(f, "'.'"),
            Uint              => write!(f, "integer"),
            HexDigits         => write!(f, "hex digits"),
            Ident             => write!(f, "idetifier"),
            Underscore        => write!(f, "'_'"),
        }
    }
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

    /// Parses a string.
    fn parse_str<Cm>(
        &mut self,
        text: &str,
        metrics: Cm,
        pattern: &str,
        token: AtmaToken)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        if text.starts_with(pattern) {
            Some((token, metrics.width(pattern)))
        } else {
            None
        }
    }

    /// Parses a Ident token.
    fn parse_ident<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let mut pos = Pos::ZERO;
        let mut col_iter = metrics.iter_columns(text);
        match col_iter.next() {
            Some((next, adv)) if next
                .chars()
                .all(|c| c.is_alphabetic() || c == '_') =>
            {
                pos += adv;
            }
            _ => return None,
        }

        while let Some((next, adv)) = col_iter.next() {
            if next.chars().all(|c| c.is_alphanumeric() || c == '_') {
                pos += adv;
            } else {
                break;
            }
        }

        Some((AtmaToken::Ident, pos))
    }

    /// Parses an Uint token.
    fn parse_uint<Cm>(&mut self, mut text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let (radix, adv) = if text.starts_with("0b") {
            text = &text[2..];
            (2, Pos::new(2, 0, 2))
        } else if text.starts_with("0o") {
            text = &text[2..];
            (8, Pos::new(2, 0, 2))
        } else if text.starts_with("0x") {
            text = &text[2..];
            (16, Pos::new(2, 0, 2))
        } else {
            // Unprefixed uints can't start with '_'.
            if text.starts_with('_') { return None; }
            (10, Pos::ZERO)
        };

        let rest = text
            .trim_start_matches(|c: char| c.is_digit(radix) || c == '_');
        let substr = &text[..text.len() - rest.len()];
        
        if !substr.is_empty() {
            return Some((AtmaToken::Uint, adv + metrics.width(substr)));
        } else {
            None
        }
    }

    /// Parses an HexDigits token.
    fn parse_hex_digits<Cm>(&mut self, mut text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let rest = text
            .trim_start_matches(|c: char| c.is_digit(16));
        let substr = &text[..text.len() - rest.len()];
        
        if !substr.is_empty() {
            return Some((AtmaToken::HexDigits, metrics.width(substr)));
        } else {
            None
        }
    }

    /// Parses a Float token.
    fn parse_float<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
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

            let next = rest.trim_start_matches(|c: char| c.is_digit(10));
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

    /// Parses a RawStringOpen token.
    fn parse_raw_string_open<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
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
    fn parse_raw_string_close<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
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
    fn parse_raw_string_text<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let mut pos = Pos::ZERO;
        let mut col_iter = metrics.iter_columns(text);

        while let Some((next, adv)) = col_iter.next() {
            if next == "\"" && self
                .parse_raw_string_close(&text[pos.byte..], metrics)
                .is_some()
            {
                self.open = Some(AtmaToken::RawStringText);
                return Some((AtmaToken::RawStringText, pos));
            } else {
                pos += adv;
            }
        }

        Some((AtmaToken::RawStringText, pos))
    }

    /// Parses a StringText token.
    fn parse_string_text<Cm>(
        &mut self,
        text: &str,
        metrics: Cm,
        open: AtmaToken)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let mut pos = Pos::ZERO;
        let mut col_iter = metrics.iter_columns(text);

        while let Some((next, adv)) = col_iter.next() {
            match (next, open) {
                ("\\", _) => match col_iter.next() {
                    Some(("\\", adv2)) |
                    Some(("\"", adv2)) |
                    Some(("'",  adv2)) |
                    Some(("t",  adv2)) |
                    Some(("r",  adv2)) |
                    Some(("n",  adv2)) => pos += (adv + adv2),
                    Some(("u",  adv2)) => unimplemented!("unicode escapes unsupported"),
                    _                  => return None,
                },
                
                ("'",  AtmaToken::StringOpenSingle) |
                ("\"", AtmaToken::StringOpenDouble) => {
                    return Some((AtmaToken::StringText, pos));
                },

                _ => pos += adv,
            }
        }

        Some((AtmaToken::StringText, pos))
    }

    /// Parses a Whitespace token.
    fn parse_whitespace<Cm>(&mut self, text: &str, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr = &text[..text.len() - rest.len()];
            Some((AtmaToken::Whitespace, metrics.width(substr)))
        } else {
            None
        }
    }
}

impl Scanner for AtmaScanner {
    type Token = AtmaToken;

    fn scan<'text, Cm>(&mut self, text: &'text str, metrics: Cm)
        -> Option<(Self::Token, Pos)>
        where Cm: ColumnMetrics,
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
                if let Some(parse) = self.parse_raw_string_close(text, metrics) {
                    self.depth = 0;
                    return Some(parse);
                }
                return_if_some!(self.parse_raw_string_text(text, metrics));
                None
            },

            Some(StringOpenSingle) => {
                return_if_some!(self.parse_str(text, metrics, "\'", StringCloseSingle));
                if let Some(parse) = self
                    .parse_string_text(text, metrics, StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                None
            },
            Some(StringOpenDouble) => {
                return_if_some!(self.parse_str(text, metrics, "\"", StringCloseDouble));
                if let Some(parse) = self
                    .parse_string_text(text, metrics, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }
                None
            },

            Some(Hash) => {
                // HexDigits can only come after Hash.
                return_if_some!(self.parse_hex_digits(text, metrics));
                self.scan(text, metrics)
            },

            Some(Colon) => {
                // Colon will make Position parts a priority until something
                // else is seen. It is important to have uint before float to
                // avoid swallowing them up with decimals.
                self.open = Some(Colon);
                return_if_some!(self.parse_uint(text, metrics));
                return_if_some!(self.parse_str(text, metrics, ".", Decimal));
                return_if_some!(self.parse_str(text, metrics, "*", Mult));
                return_if_some!(self.parse_str(text, metrics, "+", Plus));
                return_if_some!(self.parse_str(text, metrics, "-", Minus));

                self.open = None;
                self.scan(text, metrics)
            },

            None => {
                return_if_some!(self.parse_whitespace(text, metrics));
                return_if_some!(self.parse_str(text, metrics, "(", OpenParen));
                return_if_some!(self.parse_str(text, metrics, ")", CloseParen));
                return_if_some!(self.parse_str(text, metrics, "[", OpenBracket));
                return_if_some!(self.parse_str(text, metrics, "]", CloseBracket));
                return_if_some!(self.parse_str(text, metrics, "{", OpenBrace));
                return_if_some!(self.parse_str(text, metrics, "}", CloseBrace));

                // RawStringOpen must be parsed before Hash.
                if let Some(parse) = self.parse_raw_string_open(text, metrics) {
                    self.open = Some(RawStringOpen);
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_str(text, metrics, "\'", StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_str(text, metrics, "\"", StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }

                if let Some(parse) = self.parse_str(text, metrics, ":", Colon) {
                    self.open = Some(Colon);
                    return Some(parse);
                }
                return_if_some!(self.parse_str(text, metrics, ",", Comma));
                if let Some(parse) = self.parse_str(text, metrics, "#", Hash) {
                    self.open = Some(Hash);
                    return Some(parse);
                }

                return_if_some!(self.parse_str(text, metrics, "*", Mult));
                return_if_some!(self.parse_str(text, metrics, "+", Plus));
                return_if_some!(self.parse_str(text, metrics, "-", Minus));
                
                // Float must be parsed before Uint and Decimal.
                return_if_some!(self.parse_float(text, metrics));
                return_if_some!(self.parse_str(text, metrics, ".", Decimal));
                return_if_some!(self.parse_uint(text, metrics));

                // Ident must be parsed before Underscore.
                return_if_some!(self.parse_ident(text, metrics));
                return_if_some!(self.parse_str(text, metrics, "_", Underscore));
                
                None
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}

