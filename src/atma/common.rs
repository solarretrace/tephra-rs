////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common parsers.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Local imports.
use crate::atma::AtmaToken;
use crate::atma::AtmaScanner;
use crate::lexer::Lexer;
use crate::position::ColumnMetrics;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::ParseError;
use crate::result::Failure;
use crate::combinator::one;
use crate::combinator::exact;
use crate::combinator::any;
use crate::combinator::right;
use crate::combinator::empty;
use crate::combinator::bracket_dynamic;
use crate::combinator::bracket;
use crate::combinator::text;

// Standard library imports.
use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////
// Integer parsing
////////////////////////////////////////////////////////////////////////////////

pub fn uint<'text, Cm, T>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, T>
    where
        Cm: ColumnMetrics,
        T: FromStrRadix + std::fmt::Debug,
{
    lexer.next_filtered(); // Remove prefixed tokens.
    let (mut val, succ) = text(one(AtmaToken::Uint))
        (lexer)?
        .take_value();

    let radix = if val.starts_with("0b") {
        val = &val[2..];
        2
    } else if val.starts_with("0o") {
        val = &val[2..];
        8
    } else if val.starts_with("0x") {
        val = &val[2..];
        16
    } else {
        10
    };

    // Remove underscores.
    let mut val = String::from(val);
    val.retain(|c| c != '_');

    match T::from_str_radix(&*val, radix) {
        Ok(val) => Ok(succ.map_value(|_| val)),
        Err(e) => {
            let display_val = T::error_from_str_radix(&*val, radix);
            Err(Failure {
                parse_error: ParseError::new("invalid integer value")
                    .with_span(format!(
                        "value ({}) is too large for {} value",
                            T::error_from_str_radix(&*val, radix).map_or_else(
                                || val.to_string(),
                                |v| v.to_string()),
                            T::SIZE_DESCRIPTION),
                        succ.lexer.last_span()),
                lexer: succ.lexer,
                source: Some(Box::new(e)),
            })
        }
    }
}


pub trait FromStrRadix: Sized {
    const SIZE_DESCRIPTION: &'static str;

    type ErrorValue: ToString;

    fn from_str_radix(src: &str, radix: u32)
        -> Result<Self, std::num::ParseIntError>;

    fn error_from_str_radix(src: &str, radix: u32)
        -> Option<Self::ErrorValue>;
}

macro_rules! from_str_radix_impl {
    ($t:ty, $desc:expr, $ev:ty) => {
        impl FromStrRadix for $t {
            const SIZE_DESCRIPTION: &'static str = $desc;
            type ErrorValue = $ev;
            fn from_str_radix(src: &str, radix: u32)
                -> Result<$t, std::num::ParseIntError>
            {
                <$t>::from_str_radix(src, radix)
            }

            fn error_from_str_radix(src: &str, radix: u32)
                -> Option<Self::ErrorValue>
            {
                <$ev>::from_str_radix(src, radix).ok()
            }
        }
    }
}

from_str_radix_impl!(isize, "signed size",      i128);
from_str_radix_impl!(i8   , "signed 8 bit",     i128);
from_str_radix_impl!(i16  , "signed 16 bit",    i128);
from_str_radix_impl!(i32  , "signed 32 bit",    i128);
from_str_radix_impl!(i64  , "signed 64 bit",    i128);
from_str_radix_impl!(i128 , "signed 128 bit",   i128);
from_str_radix_impl!(usize, "unsigned size",    u128);
from_str_radix_impl!(u8   , "unsigned 8 bit",   u128);
from_str_radix_impl!(u16  , "unsigned 16 bit",  u128);
from_str_radix_impl!(u32  , "unsigned 32 bit",  u128);
from_str_radix_impl!(u64  , "unsigned 64 bit",  u128);
from_str_radix_impl!(u128 , "unsigned 128 bit", u128);


////////////////////////////////////////////////////////////////////////////////
// Float parsing
////////////////////////////////////////////////////////////////////////////////

pub fn float<'text, Cm, T>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, T>
    where
        Cm: ColumnMetrics,
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::error::Error + Sync + Send + 'static
{
    let (mut val, succ) = text(one(AtmaToken::Float))
        (lexer)?
        .take_value();

    
    match T::from_str(&*val) {
        Ok(val) => Ok(succ.map_value(|_| val)),
        Err(e) => Err(Failure {
            parse_error: ParseError::new("invalid float value")
                .with_span("invalid value", succ.lexer.span()),
            lexer: succ.lexer,
            source: Some(Box::new(e)),
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// String parsing
////////////////////////////////////////////////////////////////////////////////

pub fn string<'text, Cm>(
    lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Cow<'text, str>>
    where Cm: ColumnMetrics,
{
    if let Ok(succ) = raw_string(lexer.clone()) {
        return Ok(succ.map_value(Cow::from))
    }

    escaped_string(lexer)
}

pub fn raw_string<'text, Cm>(
    lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, &'text str>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    bracket(
        one(RawStringOpen),
        text(one(RawStringText)),
        one(RawStringClose))
        (lexer)
}

pub fn escaped_string<'text, Cm>(
    lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Cow<'text, str>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    let corresponding = move |lexer, tok| match tok {
        StringOpenSingle => one(StringCloseSingle)(lexer),
        StringOpenDouble => one(StringCloseDouble)(lexer),
        _ => unreachable!(),
    };

    bracket_dynamic(
        any(&[StringOpenSingle, StringOpenDouble]),
        text(one(StringText)),
        corresponding)
        (lexer)
        .map_value(unescape)
}

fn unescape<'text>(input: &'text str) -> Cow<'text, str> {
    const ESCAPES: [char; 6] = ['\\', '"', '\'', 't', 'r', 'n'];
    let mut owned: Option<String> = None;

    let mut chars = input.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            match chars.next() {
                // NOTE: These should all step by column, because
                // they're escaped text.
                Some((_, e)) if ESCAPES.contains(&e) => {
                    if owned.is_none() {
                        owned = Some(String::with_capacity(input.len()));
                        owned.as_mut().unwrap().push_str(&input[0..i]);
                    }

                    owned.as_mut().unwrap().push(match e {
                        '\\' => '\\',
                        '"'  => '"',
                        '\'' => '\'',
                        't'  => '\t',
                        'r'  => '\r',
                        'n'  => '\n',
                        _    => unreachable!(),
                    });
                },
                Some((_, 'u'))  => unimplemented!("unicode escapes unsupported"),
                // TODO: Make this an error instead.
                Some(_)    |
                None       => panic!("invalid escape character"),
            }
        } else if let Some(owned) = owned.as_mut() {
            owned.push(c);
        }
    }

    match owned {
        Some(s) => s.into(),
        None    => input.into(),
    }
}
