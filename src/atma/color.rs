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
use crate::atma::AtmaScanner;
use crate::atma::AtmaToken;
use crate::atma::FnArg;
use crate::atma::fn_call;
use crate::combinator::any;
use crate::combinator::bracket;
use crate::combinator::bracket_dynamic;
use crate::combinator::exact;
use crate::combinator::error_context;
use crate::combinator::one;
use crate::combinator::right;
use crate::combinator::seq;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::Failure;
use crate::result::ParseError;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::Success;
use crate::span::NewLine;
use crate::span::Span;

// External library imports.
use ::color::Color;
use ::color::Rgb;

// Standard library imports.
use std::str::FromStr as _;
use std::convert::TryFrom as _;



////////////////////////////////////////////////////////////////////////////////
// color
////////////////////////////////////////////////////////////////////////////////

pub fn color<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, Color>
    where Nl: NewLine,
{
    unimplemented!()
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
}


////////////////////////////////////////////////////////////////////////////////
// rgb_hex
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which parses a hex code with the given number of digits.
pub fn rgb_hex_code<'text, Nl>(lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, Rgb>
    where Nl: NewLine,
{
    let (mut val, succ) = text(exact(
            seq(&[AtmaToken::Hash, AtmaToken::HexDigits])))
        (lexer)?
        .take_value();

    if val.len() == 4 || val.len() == 7 {
        let rgb = Rgb::from_hex_code(val).unwrap();
        Ok(Success {
            lexer: succ.lexer,
            value: rgb,
        })
    } else {
        Err(Failure {
            parse_error: ParseError::new("invalid color code")
                .with_span(
                    format!("3 or 6 digits required, {} provided",
                        val.len() - 1),
                    succ.lexer.last_span()),
            lexer: succ.lexer,
            source: None,
        })
    }
}



////////////////////////////////////////////////////////////////////////////////
// color_function
////////////////////////////////////////////////////////////////////////////////

pub fn color_function<'text, Nl>(lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, Color>
    where Nl: NewLine,
{
    let (val, succ) = fn_call
            (lexer)?
        .take_value();

    if val.name.eq_ignore_ascii_case("rgb") {
        return rgb_from_args(succ.lexer, val.args)
            .map_value(Color::from);
    }

    Err(Failure {
        parse_error: ParseError::new("invalid color")
            .with_span("not a recognized color form", succ.lexer.full_span()),
        lexer: succ.lexer,
        source: None,
    })
}


fn rgb_from_args<'text, Nl>(
    lexer: Lexer<'text, AtmaScanner, Nl>,
    args: Vec<(FnArg, Span<'text, Nl>)>)
    -> ParseResult<'text, AtmaScanner, Nl, Rgb>
    where Nl: NewLine,
{
    if args.len() != 3 {
        return Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span(
                    format!("RGB requires 3 arguments, {} provided",
                        args.len()),
                    lexer.last_span()),
            lexer,
            source: None,

        });
    }

    use FnArg::*;
    match (&args[0], &args[1], &args[2]) {
        ((F32(r), rs), (F32(g), gs), (F32(b), bs)) => {
            if *r < 0.0 || *r > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span("value out of range [0.0, 1.0]", *rs),
                    lexer,
                    source: None,
                })
            } else if *g < 0.0 || *g > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span("value out of range [0.0, 1.0]", *gs),
                    lexer,
                    source: None,
                })
            } else if *b < 0.0 || *b > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span("value out of range [0.0, 1.0]", *bs),
                    lexer,
                    source: None,
                })

            } else {
                Ok(Success {
                    value: Rgb::from([*r, *g, *b]),
                    lexer,
                })
            }
        }
        
        ((U32(r), rs), (U32(g), gs), (U32(b), bs)) => match (
            u8::try_from(*r),
            u8::try_from(*g),
            u8::try_from(*b)) 
        {
            (Ok(r), Ok(g), Ok(b)) => Ok(Success {
                value: Rgb::from([r, g, b]),
                lexer,
            }),

            (Ok(r), Ok(g), Err(e)) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span("octet out of range", *bs),
                lexer,
                source: Some(Box::new(e)),
            }),
            (Ok(r), Err(e),     _) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span("octet out of range", *gs),
                lexer,
                source: Some(Box::new(e)),
            }),
            (Err(e),     _,     _) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span("octet out of range", *rs),
                lexer,
                source: Some(Box::new(e)),
            }),
        }

        ((U32(_), _), (U32(_), _), (F32(_), s)) |
        ((U32(_), _), (F32(_), s), _          ) => Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span("expected u8 value", *s),
            lexer,
            source: None,
        }),

        ((F32(_), _), (F32(_), _), (U32(_), s)) |
        ((F32(_), _), (U32(_), s), _          ) => Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span("expected f32 value", *s),
            lexer,
            source: None,
        }),
    }
}
