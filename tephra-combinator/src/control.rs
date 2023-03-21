////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser control combinators.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseError;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Spanned;
use tephra::Success;
use tephra::Failure;
use tephra::RecoverError;
use tephra::Recover;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;




////////////////////////////////////////////////////////////////////////////////
// Lexer context combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which disables error contexts.
pub fn raw<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "raw").entered();

        let mut ctx = ctx.clone()
            .locked(true);
        let _ = ctx.take_local_context();

        (parser)
            (lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
    }
}

/// A combinator which disables error recovery.
pub fn unrecoverable<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "unrecoverable").entered();

        let mut ctx = ctx.clone();
        let _ = ctx.take_error_sink();
        
        (parser)
            (lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
    }
}

/// A combinator which performs error recovery, returning a default value when
/// an error occurs.
pub fn recover_default<'text, Sc, F, V>(
    mut parser: F,
    recover: Recover<Sc::Token>)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
        V: Default
{
    if recover == Recover::Wait || recover.limit() == Some(&0) {
        // TODO: Maybe Wait is a useful state to prevent advancing?
        panic!("invalid recover state");
    }

    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "recover_default").entered();
        
        let mut base_lexer = lexer.clone();
        base_lexer.set_recover_state(recover.clone());

        match (parser)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ),

            Err(Failure { lexer, parse_error, .. }) => match ctx
                .send_error(parse_error)
            {
                Err(parse_error) => Err(Failure { lexer, parse_error }),

                Ok(()) => match base_lexer.advance_to_recover() {
                    Ok(_) => {
                        Ok(Success {
                            lexer: base_lexer,
                            value: V::default(),
                        })
                    },
                    Err(RecoverError::EndOfText) => {
                        Err(Failure {
                            parse_error: ParseError::unexpected_end_of_text(
                                base_lexer.source(),
                                base_lexer.cursor_span()),
                            lexer: base_lexer,
                        })
                    },
                    Err(RecoverError::LimitExceeded) => {
                        // Recover limit should be non-zero.
                        unreachable!()
                    },
                },
            },
        }
    }
}

/// A combinator which performs error recovery, returning a `None` value when
/// an error occurs.
pub fn recover_option<'text, Sc, F, V>(
    mut parser: F,
    recover: Recover<Sc::Token>)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
{
    let option_parser = move |lexer, ctx| {
        (parser)
            (lexer, ctx)
            .map_value(Some)
    };

    recover_default(option_parser, recover)
}

pub fn recover_until<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        let mut res = (parser)
            (lexer.clone(), ctx.clone());

        while lexer.recover_state().is_recovering() {
            match res {
                Ok(mut succ) => {
                    succ.lexer.clear_recover_state();
                    res = Ok(succ);
                },
                Err(mut fail) => match fail.lexer.advance_to_recover() {
                    Ok(_) => {
                        res = (parser)
                            (fail.lexer.clone(), ctx.clone());
                    },
                    Err(RecoverError::LimitExceeded) => {
                        fail.lexer.clear_recover_state();
                        res = Err(fail)
                    },
                    Err(RecoverError::EndOfText) => {
                        fail.lexer.clear_recover_state();
                        res = Err(Failure {
                            parse_error: ParseError::unexpected_end_of_text(
                                fail.lexer.source(),
                                fail.lexer.cursor_span()),
                            lexer: fail.lexer,
                        })
                    },
                }
            }
        }
        res
    }
}


////////////////////////////////////////////////////////////////////////////////
// Token filtering combinators.
////////////////////////////////////////////////////////////////////////////////


/// A combinator which filters tokens during exectution of the given parser.
///
/// ### Parameters
/// + `filter_fn`: A function which will return `false` for any
/// [`Scanner::Token`] to be excluded during the parse.
/// + `parser`: The parser to run with with the applied token filter.
///
/// [`Scanner::Token`]: tephra::Scanner#associatedtype.Token
pub fn filter_with<'text, Sc, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |mut lexer, ctx| {
        let _span = span!(Level::DEBUG, "filter").entered();

        let old_filter = lexer.take_filter();
        lexer.set_filter_fn(filter_fn.clone());

        match (parser)
            (lexer, ctx)
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
pub fn unfiltered<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |mut lexer, ctx| {
        let _span = span!(Level::DEBUG, "unfiltered").entered();

        event!(Level::TRACE, "before removing filter:\n{}", lexer);

        let filter = lexer.take_filter();
        match (parser)
            (lexer, ctx)
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




////////////////////////////////////////////////////////////////////////////////
// Result transforming combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, ()>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "discard").entered();

        match (parser)
            (lexer, ctx)
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
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) 
        -> ParseResult<'text, Sc, &'text str>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "text").entered();

        let start = lexer.cursor_pos().byte;
        match (parser)
            (lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                let end = succ.lexer.end_pos().byte;
                let value = &succ.lexer.source().as_str()[start..end];

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
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Spanned<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>)
            -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "spanned").entered();

        event!(Level::TRACE, "before subparse:\n{}", lexer);

        match (parser)
            (lexer.sublexer(), ctx)
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
