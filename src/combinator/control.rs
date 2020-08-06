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
use crate::span::NewLine;
use crate::span::Span;
use crate::result::ParseResult;
use crate::result::Success;
use crate::result::ParseError;
use crate::result::Failure;
use crate::result::FailureOwned;


////////////////////////////////////////////////////////////////////////////////
// Control combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which disables all token filters during exectution of the given
/// parser.
pub fn exact<'text, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    move |mut lexer| {
        let filter = lexer.take_filter();
        match (parser)(lexer) {
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

////////////////////////////////////////////////////////////////////////////////
// Parse result substitution combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'text, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
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
pub fn text<'text, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>) 
        -> ParseResult<'text, Sc, Nl, &'text str>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    move |lexer| {
        let start = lexer.end_pos().byte;
        match (parser)(lexer) {
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
pub fn with_span<'text, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>)
        -> ParseResult<'text, Sc, Nl, (V, Span<'text, Nl>)>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer) {
            Ok(succ) => {
                Ok(Success {
                    value: (succ.value, succ.lexer.span()),
                    lexer: succ.lexer,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}


/// A combinator which includes the span of the parsed value.
pub fn error_context<'text, Sc, Nl, S, F, V>(
    description: &'static str,
    span_message: S,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>)
        -> ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        S: Into<String>,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    let msg = span_message.into();
    move |lexer| {
        let start = lexer.start_pos();
        let source = lexer.source();

        match (parser)(lexer.sublexer()) {
            Ok(succ) => Ok(succ),

            // TODO: Try adding context as new highlight on existing span.
            Err(fail) => Err(Failure {
                parse_error: ParseError::new(description)
                    .with_span(msg.clone(), Span::new_enclosing(
                        start,
                        fail.lexer.end_pos(),
                        source)),
                lexer,
                source: Some(Box::new(FailureOwned::from(fail))),
            }),
        }
    }
}
