////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::Success;


////////////////////////////////////////////////////////////////////////////////
// Parse result substitution combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer) {
            Ok(succ) => {
                Ok(Success {
                    lexer: succ.lexer,
                    value: (),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}

/// A combinator which replaces a parsed value with the source text of the
/// parsed span.
pub fn text<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, &'t str>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        let start = lexer.current_pos().byte;
        match (parser)(lexer) {
            Ok(succ) => {
                let end = succ.lexer.current_pos().byte;

                let value = &succ.lexer.source()[start..end];
                Ok(Success {
                    lexer: succ.lexer,
                    value,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which sequences two parsers wich must both succeed,
/// returning the value of the first one.
pub fn left<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
{
    move |lexer| {
        let (l, succ) = (left)
            (lexer)?
            .take_value();

        (right)
            (succ.lexer)
            .map_value(|_| l)
            .consume_current_if_success()
    }
}

/// Returns a parser which sequences two parsers wich must both succeed,
/// returning the value of the second one.
pub fn right<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
{
    move |lexer| {
        let succ = (left)
            (lexer)?;

        (right)
            (succ.lexer)
            .consume_current_if_success()
    }
}

/// Returns a parser which sequences two parsers wich must both succeed,
/// returning their values in a tuple.
pub fn both<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, (X, Y)>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
{
    move |lexer| {
        let (l, succ) = (left)
            (lexer)?
            .take_value();

        (right)
            (succ.lexer)
            .map_value(|r| (l, r))
            .consume_current_if_success()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tolerance & inversion combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Option<V>>
    where
        Sc: Scanner,
        Nl: Clone,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer.clone()) {
            Ok(succ) => Ok(succ.map_value(Some)),
            Err(_fail) => Ok(Success {
                    lexer: lexer,
                    value: None,
            }),
        }

    }
}

// require_if


////////////////////////////////////////////////////////////////////////////////
// Repetition combinators.
////////////////////////////////////////////////////////////////////////////////

// pub fn intersperse_collect<'t, Sc, Nl, F, G, V, U>(
//     mut parser: F,
//     mut inter_parser: G)
//     -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Vec<V>>
//     where
//         Sc: Scanner,
//         F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
//         G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
// {
//     move |lexer| {
        
//     }
// }

// intersperse_until
// intersperse
// repeat_collect
// repeat_until
// repeat

// circumfix
// bracket
// bracket_with
