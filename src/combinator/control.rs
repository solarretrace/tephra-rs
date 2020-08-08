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


////////////////////////////////////////////////////////////////////////////////
// Control combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which filters tokens during exectution of the given parser.
pub fn filter<'text, Sc, Nl, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    move |mut lexer| {
        let old_filter = lexer.take_filter();
        lexer.set_filter_fn(filter_fn.clone());
        match (parser)(lexer) {
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

/// A combinator which identifies a delimiter or bracket which starts a new
/// failure span section.
pub fn section<'text, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer.sublexer()) {
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
        match (parser)(lexer.sublexer()) {
            Ok(succ) => {
                Ok(Success {
                    value: (succ.value, succ.lexer.full_span()),
                    lexer: lexer.join(succ.lexer),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}

