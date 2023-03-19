////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Alternative combinators.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::fail;
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;

// External library imports.
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// atomic
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts a parse such that if any prefix of it
/// succeeds, the entire parse must succeed.
/// 
/// Note that filtered tokens are not counted as prefix tokens.
pub fn atomic<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "atomic").entered();

        // TODO: Remove filtered prefix tokens.
        let start = lexer.cursor_pos();

        match (parser)
            (lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                Ok(Success {
                    lexer: succ.lexer,
                    value: Some(succ.value),
                })
            },
            Err(fail) => if start == fail.lexer.cursor_pos() {
                Ok(Success {
                    lexer: fail.lexer,
                    value: None,
                })
            } else {
                Err(fail)
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// implies
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers which must both succeed if the
/// first succeeds, returning their values in a tuple.
pub fn implies<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<(X, Y)>>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "implies").entered();


        let (l, succ) = (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        (right)
            (succ.lexer, ctx)
            .trace_result(Level::TRACE, "right capture")
            .map_value(|r| Some((l, r)))
    }
}


////////////////////////////////////////////////////////////////////////////////
// cond
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts a parse only if the given predicate is true.
///
/// This acts like a `maybe` combinator that can be conditionally disabled:
/// `cond(|| false, p)` is identical to `maybe(p)` and 
/// `cond(|| true, p)` is identical to `p`.
pub fn cond<'text, Sc, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "cond").entered();

        if pred() {
            parser(lexer, ctx)
                .trace_result(Level::TRACE, "true branch")
        } else {
            fail(lexer, ctx)
        }
    }
}
