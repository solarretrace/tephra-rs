////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! AtmaExpr lexer and parser definitions.
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
use crate::result::Reason;
use crate::result::Failure;
use crate::primitive::one;
use crate::combinator::right;
use crate::combinator::text;

// Standard library imports.
use std::convert::TryInto as _;


////////////////////////////////////////////////////////////////////////////////
// Scanner
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

    /// A binary integer prefix marker "0b".
    IntPrefixBin,
    /// A octal integer prefix marker "0o".
    IntPrefixOct,
    /// A hexidecimal integer prefix marker "0x".
    IntPrefixHex,
    
    /// Any number of hexidecimal integer digits or underscore characters.
    IntDigits,
    /// An identifier with the form "[_[alpha]][alphanumeric]+".
    Ident,

    /// An underscore character '_'.
    Underscore,
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtmaExprScanner {
    open: Option<AtmaToken>,
    raw_string_bracket_count: u64,
}

impl AtmaExprScanner {
    pub fn new() -> Self {
        AtmaExprScanner {
            open: None,
            raw_string_bracket_count: 0,
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

    /// Parses a IntPrefixBin token.
    fn parse_int_prefix_bin(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("0b") {
            Some((AtmaToken::IntPrefixBin, Pos::new(2, 0, 2)))
        } else {
            None
        }
    }

    /// Parses a IntPrefixOct token.
    fn parse_int_prefix_oct(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("0o") {
            Some((AtmaToken::IntPrefixOct, Pos::new(2, 0, 2)))
        } else {
            None
        }
    }

    /// Parses a IntPrefixHex token.
    fn parse_int_prefix_hex(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("0x") {
            Some((AtmaToken::IntPrefixHex, Pos::new(2, 0, 2)))
        } else {
            None
        }
    }

    /// Parses a IntDigits token.
    fn parse_int_digits(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        // Can't start with '_'.
        if text.starts_with('_') { return None; }

        let rest = text
            .trim_start_matches(|c: char| c.is_digit(16) || c == '_');
        
        let cols = text.len() - rest.len();
        if cols > 0 {
            return Some((AtmaToken::IntDigits, Pos::new(cols, 0, cols)));
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
                    self.raw_string_bracket_count += 1;
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

impl Scanner for AtmaExprScanner {
    type Token = AtmaToken;
    type Error = TokenError;

    fn lex_prefix_token<'text, Nl>(&mut self, text: &'text str)
        -> Result<(Self::Token, Pos), (Self::Error, Pos)>
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
                if let Some(parse) = self.parse_raw_string_text::<Nl>(text) {
                    return Ok(parse);
                }
                Err((TokenError("Non-terminated raw string"), Pos::ZERO))
            },

            Some(StringOpenSingle) => {
                if let Some(parse) = self.parse_string_close_single(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text::<Nl>(text, StringOpenSingle)
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
                    .parse_string_text::<Nl>(text, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Ok(parse);
                }
                Err((TokenError("Unrecognized string escape"),
                    Pos::new(1, 0, 1)))
            },

            None => {
                if let Some(parse) = self.parse_whitespace::<Nl>(text) {
                    return Ok(parse);
                }

                if let Some(parse) = self.parse_open_paren(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_close_paren(text) {
                    return Ok(parse);
                }

                // RawStringOpen must be parsed before Hash.
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

                if let Some(parse) = self.parse_colon(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_comma(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_hash(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_mult(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_plus(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_minus(text) {
                    return Ok(parse);
                }
                
                // Float must be parsed before IntDigits and Decimal.
                if let Some(parse) = self.parse_float(text) {
                    return Ok(parse);
                }

                if let Some(parse) = self.parse_decimal(text) {
                    return Ok(parse);
                }

                // Int prefixes must be parsed before IntDigits.
                if let Some(parse) = self.parse_int_prefix_bin(text) {
                    return Ok(parse);
                }

                if let Some(parse) = self.parse_int_prefix_oct(text) {
                    return Ok(parse);
                }

                if let Some(parse) = self.parse_int_prefix_hex(text) {
                    return Ok(parse);
                }


                if let Some(parse) = self.parse_int_digits(text) {
                    return Ok(parse);
                }
                // Ident must be parsed before Underscore.
                if let Some(parse) = self.parse_ident(text) {
                    return Ok(parse);
                }

                if let Some(parse) = self.parse_underscore(text) {
                    return Ok(parse);
                }
                
                Err((TokenError("Unrecognized token"), Pos::new(1, 0, 1)))
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// Values
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellRef {
    Index(u32),
    Position(u16, u16, u16),
    Group(String, usize),
    Name(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellSelector {
    All,
    Index(u32),
    PositionSelector(Option<u16>, Option<u16>, Option<u16>),
    Group(String, usize),
    GroupAll(String),
    Name(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    name: String,
    args: Vec<FunctionArg>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionArg {
    CellRef(CellRef),
    Color(Color),
    Channel(Channel),
    Function(Function),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Rgb,
    Hsv,
    Hsl,
}



pub fn parse_color<'text, Nl>(mut lexer: Lexer<'text, AtmaExprScanner, Nl>)
    -> ParseResult<'text, AtmaExprScanner, Nl, Color>
    where Nl: NewLine,
{
    let filter = lexer.take_filter();
    match right(
            one(AtmaToken::Hash),
            text(one(AtmaToken::IntDigits)))
        (lexer)
    {
        Ok(mut succ)  => {
            use std::str::FromStr;
            succ.lexer.set_filter(filter);
            if succ.value.len() != 6 {
                return Err(Failure {
                    lexer: succ.lexer,
                    reason: Reason::IncompleteParse { 
                        context: "Color requires 6 hex digits".into(),
                    },
                    source: None,
                })
            }
            match u32::from_str(succ.value) {
                Ok(val) => Ok(succ.map_value(|_| Color(val))),
                Err(e) => Err(Failure {
                    lexer: succ.lexer,
                    reason: Reason::IncompleteParse { 
                        context: "Invalid color conversion".into(),
                    },
                    source: Some(Box::new(e)),
                }),
            }
        }
        Err(mut fail) => {
            fail.lexer.set_filter(filter);
            Err(fail)
        },
    }
}
