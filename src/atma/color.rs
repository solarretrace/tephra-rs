////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Color parsing.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Local imports.
use crate::atma::AtmaToken;
use crate::atma::AtmaScanner;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::ParseError;
use crate::result::Failure;
use crate::combinator::one;
use crate::combinator::any;
use crate::combinator::bracket_dynamic;
use crate::combinator::bracket;
use crate::combinator::text;
use crate::combinator::exact;
use crate::combinator::right;

// External library imports.
use ::color::Color;


/// Returns a parser which parses a hex code with the given number of digits.
pub fn hex_code<'t, Sc, Nl>(digits: u32)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, u32>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    move |mut lexer| {
        unimplemented!()
    }
}

// pub fn color<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
//     -> ParseResult<'text, AtmaScanner, Nl, Color>
//     where Nl: NewLine,
// {
//     match exact(
//         right(
//             one(AtmaToken::Hash),
//             text(one(AtmaToken::Uint))))
//         (lexer)
//     {
//         Ok(mut succ)  => {
//             use std::str::FromStr;
//             if succ.value.len() != 6 {
//                 return Err(Failure {
//                     parse_error: ParseError::new("invalid color")
//                         .with_span("color requires 6 hex digits",
//                             succ.lexer.span()),
//                     lexer: succ.lexer,
//                     source: None,
//                 })
//             }
//             match u32::from_str(succ.value) {
//                 Ok(val) => Ok(succ.map_value(|_| Color(val))),
//                 Err(e) => Err(Failure {
//                     parse_error: ParseError::new("invalid color")
//                         .with_span("color conversion failed",
//                             succ.lexer.span()),
//                     lexer: succ.lexer,
//                     source: Some(Box::new(e)),
//                 }),
//             }
//         }
//         Err(fail) => Err(fail),
//     }
// }
