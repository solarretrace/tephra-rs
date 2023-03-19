////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Alternative combinators.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;
use tephra_tracing::Level;
use tephra_tracing::span;




////////////////////////////////////////////////////////////////////////////////
// either
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts each of the given parsers in
/// sequence, returning the first which succeeds.
pub fn either<'text, Sc, L, R, X>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "either").entered();

        let lexer_start = lexer.clone();
        
        (left)(lexer, ctx.clone())
            .trace_result(Level::TRACE, "left branch")
            .or_else(|_| (right)(lexer_start, ctx)
                .trace_result(Level::TRACE, "right branch"))

        // TODO: Better error handling?
    }
}



////////////////////////////////////////////////////////////////////////////////
// maybe
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "maybe").entered();

        match (parser)(lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                Ok(Success {
                    lexer: succ.lexer,
                    value: Some(succ.value),
                })
            },
            Err(fail) => {
                Ok(Success {
                    lexer: fail.lexer,
                    value: None,
                })
            },
        }
    }
}


/// Returns a parser which requires a parse to succeed if the given
/// predicate is true.
///
/// This acts like a `maybe` combinator that can be conditionally disabled:
/// `maybe_if(|| false, p)` is identical to `maybe(p)` and 
/// `maybe_if(|| true, p)` is identical to `p`.
pub fn maybe_if<'text, Sc, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "maybe_if").entered();

        if pred() {
            parser(lexer, ctx)
                .trace_result(Level::TRACE, "true branch")
                .map_value(Some)
        } else {
            maybe(&mut parser)
                (lexer, ctx)
                .trace_result(Level::TRACE, "false branch")
        }
    }
}
