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
use tephra::ParseResultExt as _;
use tephra::Recover;
use tephra::Scanner;
use tephra::Success;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;

// Standard library imports.
use std::rc::Rc;


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
        let _trace_span = span!(Level::DEBUG, "~raw").entered();

        let mut ctx = ctx.clone()
            .locked(true);
        let _ = ctx.take_local_context();
        event!(Level::TRACE, "error contexts disabled");

        (parser)
            (lexer, ctx)
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
        let _trace_span = span!(Level::DEBUG, "~unrec").entered();

        let mut ctx = ctx.clone();
        let _ = ctx.take_error_sink();
        event!(Level::TRACE, "error recovery disabled");
        
        (parser)
            (lexer, ctx)
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
        let _trace_span = span!(Level::DEBUG, "~rec_").entered();

        let mut base_lexer = lexer.clone();
        base_lexer.set_recover_state(Some(Rc::clone(&recover)));

        match (parser)
            (lexer, ctx.clone())
        {
            Ok(succ) => Ok(succ),

            Err(fail) if fail.is_recoverable() => match ctx.send_error(fail) {
                Err(fail) => {
                    event!(Level::TRACE, "error recovery failed: disabled");
                    Err(fail)
                },

                Ok(()) => match base_lexer.advance_to_recover() {
                    Ok(_) => {
                        event!(Level::DEBUG, "error recovery point found ({})",
                            base_lexer.cursor_pos());
                        Ok(Success {
                            lexer: base_lexer,
                            value: V::default(),
                        })
                    },
                    Err(recover_error) => {
                        event!(Level::DEBUG, "error recovery failed: \
                            unable to find recovery point ({})",
                            base_lexer.cursor_pos());
                        Err(Box::new(recover_error))
                    },
                },
            },

            Err(fail) => {
                event!(Level::DEBUG, "error recovery failed: unrecoverable \
                    error type");
                Err(fail)
            },
        }
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
    move |mut lexer, ctx| {
        let _trace_span = span!(Level::DEBUG, "~sta").entered();

        let mut res = (parser)
            (lexer.clone(), ctx.clone());

        #[cfg_attr(not(feature="tracing"), allow(unused_variables))]
        for attempt in 0u32.. {
            let _trace_span = span!(Level::DEBUG, "", attempt).entered();
            match res {
                Ok(mut succ) => {
                    event!(Level::DEBUG, "stable after {} attempt(s)", attempt);
                    succ.lexer.set_recover_state(None);
                    return Ok(succ);
                },
                Err(fail) if fail.is_recoverable() => {
                    match lexer.advance_to_recover() {
                        Ok(_) => {
                            event!(Level::DEBUG, "error recovery point found \
                                ({})",
                                lexer.cursor_pos());

                            res = unrecoverable(&mut parser)
                                (lexer.clone(), ctx.clone());
                        },
                        Err(recover_error) => {
                            event!(Level::DEBUG, "error recovery failed: \
                                unable to find recovery point ({})",
                                lexer.cursor_pos());
                            return Err(Box::new(recover_error))
                        },
                    }
                },
                Err(fail) => { return Err(fail); },
            }
        }
        panic!("overflow on error recovery attempts");
    }
}


////////////////////////////////////////////////////////////////////////////////
// Token filtering combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which filters tokens during exectution of the given parser.
///
/// ### Parameters
/// + `filter`: A function which will return `false` for any
/// [`Scanner::Token`] to be excluded during the parse.
/// + `parser`: The parser to run with with the applied token filter.
///
/// ### Error recovery
///
/// No error recovery is attempted.
///
/// [`Scanner::Token`]: tephra::Scanner#associatedtype.Token
pub fn filter_with<'text, Sc, F, P, V>(filter: F, mut parser: P)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, V>
    where
        Sc: Scanner,
        F: for<'a> Fn(&'a Sc::Token) -> bool + 'static,
        P: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    #![allow(trivial_casts)]
    let filter = Rc::new(filter) as Rc<dyn for<'a> Fn(&'a Sc::Token) -> bool>;
    move |mut lexer, ctx| {
        let _trace_span = span!(Level::TRACE, "~filt_").entered();

        let old_filter = lexer.set_filter(Some(Rc::clone(&filter)));
        event!(Level::TRACE, "new lexer filter applied");

        (parser)
            (lexer, ctx)
            .map(|mut succ| {
                event!(Level::TRACE, "lexer filter restored");
                let _ = succ.lexer.set_filter(old_filter);
                succ
            })
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
        let _trace_span = span!(Level::TRACE, "~unfilt").entered();

        let filter = lexer.set_filter(None);
        event!(Level::TRACE, "lexer filter disabled");

        (parser)
            (lexer, ctx)
            .map(|mut succ| {
                event!(Level::TRACE, "lexer filter enabled");
                let _ = succ.lexer.set_filter(filter);
                succ
            })
    }
}


