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

// External library imports.
use tracing::Level;
use tracing::span;
use tracing::event;


////////////////////////////////////////////////////////////////////////////////
// Tolerance & inversion combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "maybe");
        let _enter = span.enter();

        let initial = lexer.clone();
        
        match parser
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ.map_value(Some)),

            Err(_)   => Ok(Success {
                value: None,
                lexer: initial,
            }),
        }
    }
}

/// Returns a parser which converts a failure into an empty success if no
/// non-filtered tokens are consumed.
///
/// This is equivalent to `maybe` if the parser consumes at most a single token.
pub fn atomic<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        V: std::fmt::Debug
{
    move |lexer| {
        let span = span!(Level::DEBUG, "atomic");
        let _enter = span.enter();

        let sub = lexer.sublexer();
        let current_cursor = lexer.cursor_pos();

        event!(Level::TRACE, "before parse:\n{}", sub);

        match parser
            (sub)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ.map_value(Some)),
            
            Err(fail) if fail.lexer.cursor_pos() > current_cursor => Err(fail),

            Err(_) => Ok(Success {
                value: None,
                lexer,
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
pub fn require_if<'text, Sc, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "require_if");
        let _enter = span.enter();

        if pred() {
            parser(lexer)
                .trace_result(Level::TRACE, "true branch")
                .map_value(Some)
        } else {
            maybe(&mut parser)
                (lexer)
                .trace_result(Level::TRACE, "false branch")
        }
    }
}
