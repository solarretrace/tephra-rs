////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser control combinators.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::maybe;

// External library imports.
use tephra::Lexer;
use tephra::Scanner;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Spanned;
use tephra::Success;
use tephra::Failure;
use tephra::ParseError;
use tephra_tracing::Level;
use tephra_tracing::span;
use tephra_tracing::event;


////////////////////////////////////////////////////////////////////////////////
// Context controls
////////////////////////////////////////////////////////////////////////////////

/// A combinator which identifies a new span context.
pub fn context<'text, Sc, F, V>(
    context: &'static str,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "context", context).entered();

        let sublexer = lexer.sublexer();

        match (parser)
            (sublexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(mut succ) => {
                succ.lexer = lexer.join(succ.lexer);
                Ok(succ)
            },
            Err(fail) => {
                let section_error = ParseError::new("parse error")
                    .with_span(
                        format!("during this {}", context),
                        lexer.clone()
                            .join(fail.lexer.clone())
                            .parse_span(),
                        lexer.column_metrics());
                Err(fail.with_context(section_error))
            },
        }
    }
}

/// Returns a parser which converts a failure into an empty success if no
/// non-filtered tokens are consumed.
///
/// This is equivalent to `context("label", maybe(..))` if the parser consumes 
/// at most a single token.
pub fn context_commit<'text, Sc, F, V>(
    context: &'static str,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "context_commit", context).entered();

        let sublexer = lexer.sublexer();
        let current_cursor = lexer.cursor_pos();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        match parser
            (sublexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ.map_value(Some)),
            
            Err(fail) if fail.lexer.cursor_pos() > current_cursor => {
                let section_error = ParseError::new("parse error")
                    .with_span(
                        format!("during this {}", context),
                        lexer.clone()
                            .join(fail.lexer.clone())
                            .parse_span(),
                        lexer.column_metrics());
                Err(fail.with_context(section_error))
            },

            Err(_) => Ok(Success {
                lexer,
                value: None,
            }),
        }
    }
}



pub fn recover_at<'text, Sc, F, V>(
    token: Sc::Token,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "recover_at", context).entered();

        match parser
            (lexer)
        {
            Ok(succ) => Ok(succ.map_value(Some)),
            Err(Failure { mut lexer, parse_error }) => {
                lexer.send(parse_error).expect("Send error to sink");
                lexer.advance_to(token.clone());
                Ok(Success { lexer, value: None })
            }
        }
    }
}

pub fn recover_after<'text, Sc, F, V>(
    token: Sc::Token,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "recover_at", context).entered();

        match parser
            (lexer)
        {
            Ok(succ) => Ok(succ.map_value(Some)),
            Err(Failure { mut lexer, parse_error }) => {
                lexer.send(parse_error).expect("Send error to sink");
                lexer.advance_past(token.clone());
                Ok(Success { lexer, value: None })
            }
        }
    }
}

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
/// [`Scanner::Token`]: tephra::lexer::Scanner#associatedtype.Token
pub fn filter_with<'text, Sc, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |mut lexer| {
        let _span = span!(Level::DEBUG, "filter").entered();

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
        let _span = span!(Level::DEBUG, "exact").entered();

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


/// Returns a parser which requires a parse to succeed if the given
/// predicate is true.
///
/// This acts like a `maybe` combinator that can be zcpconditionally disabled:
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
        let _span = span!(Level::DEBUG, "require_if").entered();

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
        let _span = span!(Level::DEBUG, "discard").entered();

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
        let _span = span!(Level::DEBUG, "text").entered();

        let start = lexer.cursor_pos().byte;
        match (parser)
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                let end = succ.lexer.end_pos().byte;
                let value = &succ.lexer.source_text()[start..end];

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
        let _span = span!(Level::DEBUG, "spanned").entered();

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
