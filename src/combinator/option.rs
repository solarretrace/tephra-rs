////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for optional values.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::Success;
use crate::span::NewLine;



////////////////////////////////////////////////////////////////////////////////
// Tolerance & inversion combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Option<V>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer.clone()) {
            Ok(succ) => Ok(succ.map_value(Some)),
            Err(_fail) => Ok(Success {
                    lexer,
                    value: None,
            }),
        }
    }
}

/// Returns a parser which requires a parse to succeed if the given
/// predicate is true.
///
/// This acts like a `maybe` combinator that can be conditionally disabled:
/// `require_if(|| false, p)` is identical to `maybe(p)` and 
/// `require_if(|| true, p)` is identical to `p`.
pub fn require_if<'t, Sc, Nl, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Option<V>>
    where
        Sc: Scanner,
        Nl: NewLine,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        if (pred)() {
            (parser)(lexer)
                .map_value(Some)
        } else {
            maybe(&mut parser)(lexer)
        }
    }
}

