////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser control combinators.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::Spanned;
use crate::result::Success;

// External library imports.
use tracing::Level;
use tracing::span;
use tracing::event;


////////////////////////////////////////////////////////////////////////////////
// Control combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which filters tokens during exectution of the given parser.
pub fn filter<'text, Sc, Cm, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |mut lexer| {
        let span = span!(Level::DEBUG, "filter");
        let _enter = span.enter();

        let old_filter = lexer.take_filter();
        lexer.set_filter_fn(filter_fn.clone());
        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(mut succ)  => {
                succ.lexer.set_filter(old_filter);
                Ok(succ)
            },
            Err(mut fail) => {
                fail.lexer.set_filter(old_filter);
                Err(fail)
            },
        }
    }
}

/// A combinator which disables all token filters during exectution of the given
/// parser.
pub fn exact<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |mut lexer| {
        let span = span!(Level::DEBUG, "exact");
        let _enter = span.enter();

        let filter = lexer.take_filter();
        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(mut succ)  => {
                succ.lexer.set_filter(filter);
                Ok(succ)
            },
            Err(mut fail) => {
                fail.lexer.set_filter(filter);
                Err(fail)
            },
        }
    }
}

/// A combinator which identifies a delimiter or bracket which starts a new
/// failure span section.
pub fn section<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "section");
        let _enter = span.enter();

        match (parser)
            (lexer.sublexer())
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(mut succ) => {
                succ.lexer = lexer.join(succ.lexer);
                Ok(succ)
            },
            Err(fail) => Err(fail),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse result substitution combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, ()>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "discard");
        let _enter = span.enter();

        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
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
/// parsed span (including any filtered prefix.)
pub fn text_exact<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) 
        -> ParseResult<'text, Sc, Cm, &'text str>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "text_exact");
        let _enter = span.enter();

        let start = lexer.end_pos().byte;
        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                let end = succ.lexer.end_pos().byte;
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

/// A combinator which replaces a parsed value with the source text of the
/// parsed span.
pub fn text<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>) 
        -> ParseResult<'text, Sc, Cm, &'text str>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |mut lexer| {
        let span = span!(Level::DEBUG, "text");
        let _enter = span.enter();

        lexer.filter_next();
        let start = lexer.end_pos().byte;
        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                let end = succ.lexer.end_pos().byte;
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


/// A combinator which includes the span of the parsed value.
pub fn spanned<'text, Sc, Cm, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Cm>)
        -> ParseResult<'text, Sc, Cm, Spanned<'text, V>>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "spanned");
        let _enter = span.enter();

        match (parser)
            (lexer.sublexer())
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                Ok(Success {
                    value: Spanned {
                        value: succ.value,
                        span: succ.lexer.full_span(),
                    },
                    lexer: lexer.join(succ.lexer),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}
