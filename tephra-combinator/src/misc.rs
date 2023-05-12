////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Miscellaneous combinators.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Span;
use tephra::Spanned;
use tephra::Success;
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Result transforming combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which applies the given `map_fn` to a parsed value.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn map<'text, Sc, P, V, F, X>(mut parser: P, mut map_fn: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        P: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
        F: FnMut(V) -> X,
{
    move |lexer, ctx| {
        (parser)
            (lexer, ctx)
            .map_value(&mut map_fn)
    }
}


/// A combinator which discards a parsed value, replacing it with `()`.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn discard<'text, Sc, F, V>(parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, ()>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    map(parser, |_| ())
}

/// A combinator which replaces a parsed value with the source text of the
/// parsed span.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn text<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) 
        -> ParseResult<'text, Sc, &'text str>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    move |mut lexer, ctx| {
        let _ = lexer.peek();
        let start = lexer.peek_token_span()
            .unwrap_or(Span::at(lexer.token_span().end()))
            .start().byte;

        match (parser)
            (lexer, ctx)
        {
            Ok(succ) => {
                let end = succ.lexer.parse_span().end().byte;
                let value = &succ.lexer.source_text().text()[start..end];

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
// Span manipulation & saving combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which performs a sub-parse (or sub-lex,) ensuring emitted spans
/// are clipped before the start of the given parser.
///
/// This effectively runs the given parser as a new parse. An filtered tokens at
/// the start of the sub-parse will also be omitted.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn sub<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _trace_span = span!(Level::TRACE, "sub").entered();
        (parser)(lexer.into_sublexer(), ctx)
    }
}

/// A combinator which includes the span of the parsed value.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn spanned<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Spanned<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    move |mut lexer, ctx| {
        let _ = lexer.peek();
        let start = lexer.peek_token_span()
            .unwrap_or(Span::at(lexer.token_span().end()))
            .start();
        
        match (parser)
            (lexer, ctx)
        {
            Ok(succ) => {
                let end = succ.lexer.parse_span().end();
                Ok(Success {
                    value: Spanned {
                        value: succ.value,
                        span: Span::enclosing(start, end),
                    },
                    lexer: succ.lexer,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}
