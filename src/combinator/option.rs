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
use crate::position::ColumnMetrics;

// External library imports.
use tracing::Level;
use tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Tolerance & inversion combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Option<V>>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
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
                lexer: initial,
                value: None,
            }),
        }
    }
}

/// Returns a parser which converts a failure into an empty success if no
/// non-filtered tokens are consumed.
///
/// This is equivalent to `maybe` if the parser consumes at most a single token.
pub fn atomic<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Option<V>>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
        V: std::fmt::Debug
{
    move |lexer| {
        let span = span!(Level::DEBUG, "atomic");
        let _enter = span.enter();

        let initial = lexer.clone();
        let end = lexer.end_pos();

        match parser
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ.map_value(Some)),
            
            Err(fail) if fail.lexer.token_span().start() > end => Err(fail),

            Err(_)   => Ok(Success {
                lexer: initial,
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
pub fn require_if<'text, Sc, Cm, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Option<V>>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
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
