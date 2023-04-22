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
use tephra::Recover;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;




////////////////////////////////////////////////////////////////////////////////
// Lexer context combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which disables error contexts.
pub fn raw<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>
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
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>
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
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
        V: Default
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "recover_default").entered();
        
        let mut base_lexer = lexer.clone();
        base_lexer.set_recover_state(Some(recover.clone()));

        match (parser)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ),

            Err(Failure { lexer, parse_error, .. }) => match ctx
                .send_error(parse_error, &lexer)
            {
                Err(parse_error) => Err(Failure { lexer, parse_error }),

                Ok(()) => match base_lexer.advance_to_recover() {
                    Ok(_) => {
                        Ok(Success {
                            lexer: base_lexer,
                            value: V::default(),
                        })
                    },
                    Err(recover_error_msg) => {
                        Err(Failure {
                            parse_error: ParseError::new(
                                base_lexer.source_text(),
                                recover_error_msg),
                            lexer: base_lexer,
                        })
                    },
                },
            },
        }
    }
}

/// A combinator which performs error recovery, returning a default value when
/// an error occurs. The recovery token is required when the combinator is
/// called rather than when it is constructed.
pub fn recover_default_delayed<'text, Sc, F, V>(
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>, Recover<Sc::Token>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
        V: Default
{
    move |lexer, ctx, recover| {
        recover_default(&mut parser, recover)(lexer, ctx)
    }
}

/// A combinator which performs error recovery, returning a `None` value when
/// an error occurs.
pub fn recover<'text, Sc, F, V>(
    mut parser: F,
    recover: Recover<Sc::Token>)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>
{
    let option_parser = move |lexer, ctx| {
        (parser)
            (lexer, ctx)
            .map_value(Some)
    };

    recover_default(option_parser, recover)
}

/// A combinator which performs error recovery, returning a `None` value when
/// an error occurs. The recovery token is required when the combinator is
/// called rather than when it is constructed.
pub fn recover_delayed<'text, Sc, F, V>(
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>, Recover<Sc::Token>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>
{
    
    move |lexer, ctx, recover| {
        let option_parser = |lexer, ctx| {
            (parser)
                (lexer, ctx)
                .map_value(Some)
        };

        recover_default(option_parser, recover)(lexer, ctx)
    }
}

/// A combinator which ends error recovery if a successful parse is achieved, or
/// resumes error recovery if a failure occurs.
pub fn stabilize<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        let mut res = (parser)
            (lexer, ctx.clone());

        loop {
            match res {
                Ok(mut succ) => {
                    succ.lexer.set_recover_state(None);
                    return Ok(succ);
                },
                Err(mut fail) => match fail.lexer.advance_to_recover() {
                    Err(recover_error_msg) => {
                        return Err(Failure {
                            parse_error: ParseError::new(
                                fail.lexer.source_text(),
                                recover_error_msg),
                            lexer: fail.lexer,
                        })
                    },
                    Ok(_) => {
                        res = (parser)
                            (fail.lexer, ctx.clone());
                    },
                },
            }
        }
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
/// ### Error recovery
///
/// No error recovery is attempted.
///
/// [`Scanner::Token`]: tephra::Scanner#associatedtype.Token
pub fn filter_with<'text, Sc, F, P, V>(filter_fn: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: for<'a> Fn(&'a Sc::Token) -> bool + Clone + 'static,
        P: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
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
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn unfiltered<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
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
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn discard<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, ()>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
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
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "text").entered();

        let start = lexer.cursor_pos().byte;
        match (parser)
            (lexer, ctx)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => {
                let end = succ.lexer.end_pos().byte;
                let value = &succ.lexer.source_text().as_str()[start..end];

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
