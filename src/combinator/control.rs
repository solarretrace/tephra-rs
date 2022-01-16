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
///
/// ### Parameters
/// + `filter_fn`: A function which will return `false` for any
/// [`Scanner::Token`] to be excluded during the parse.
/// + `parser`: The parser to run with with the applied token filter.
///
/// [`Scanner::Token`]: crate::lexer::Scanner#associatedtype.Token
pub fn filter_with<'text, Sc, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
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
///
/// ### Parameters
/// + `parser`: The parser to run without a token filter.
pub fn exact<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |mut lexer| {
        let span = span!(Level::DEBUG, "exact");
        let _enter = span.enter();

        event!(Level::TRACE, "before removing filter:\n{}", lexer);

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
pub fn section<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
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
pub fn discard<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, ()>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
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
/// parsed span.
pub fn text<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) 
        -> ParseResult<'text, Sc, &'text str>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "text");
        let _enter = span.enter();

        let start = lexer.cursor_pos().byte;
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
pub fn spanned<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>)
        -> ParseResult<'text, Sc, Spanned<'text, V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "spanned");
        let _enter = span.enter();

        event!(Level::TRACE, "before subparse:\n{}", lexer);

        match (parser)
            (lexer.sublexer())
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                Ok(Success {
                    value: Spanned {
                        value: succ.value,
                        span: succ.lexer.parse_span(),
                    },
                    lexer: lexer.join(succ.lexer),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}
