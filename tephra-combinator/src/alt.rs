////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Alternative and conditional combinators.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::map;
use crate::unrecoverable;

// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;




////////////////////////////////////////////////////////////////////////////////
// either
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts each of the given parsers in
/// sequence, returning the first which succeeds.
pub fn either<'text, Sc, L, R, X>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
{
    move |lexer, ctx| {
        let lexer_start = lexer.clone();
        
        (left)(lexer, ctx.clone())
            .or_else(|_| (right)(lexer_start, ctx))

        // TODO: Better error handling?
    }
}



////////////////////////////////////////////////////////////////////////////////
// maybe
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>
{
    move |lexer, ctx| {
        match unrecoverable(&mut parser)(lexer.clone(), ctx)
        {
            Ok(succ) => {
                Ok(Success {
                    lexer: succ.lexer,
                    value: Some(succ.value),
                })
            },
            Err(_e) => {
                event!(Level::TRACE, "maybe None: ({})", _e);
                Ok(Success {
                    lexer: lexer,
                    value: None,
                })
            },
        }
    }
}


/// Returns a parser which requires a parse to succeed if the given
/// predicate is true.
///
/// This acts like a `maybe` combinator that can be conditionally disabled:
/// `maybe_if(|| false, p)` is identical to `maybe(p)` and 
/// `maybe_if(|| true, p)` is identical to `p`.
pub fn maybe_if<'text, Sc, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let branch = (pred)();
        event!(Level::TRACE, "maybe_if: branch={}", branch);
        if branch {
            parser(lexer, ctx)
                .map_value(Some)
        } else {
            maybe(&mut parser)
                (lexer, ctx)
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// cond
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts `parser` only if the predicate `pred` is
/// satisfied.
///
/// # Similar combinators
///
/// This combinator is similar to `maybe(P)` except that the predicate can be
/// used force a `Some` or `None` result value.
pub fn cond<'text, Sc, P, F, V>(mut pred: P, mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        P: FnMut() -> bool,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        let branch = (pred)();
        event!(Level::TRACE, "cond: branch={}", branch);
        if branch {
            parser(lexer, ctx)
                .map_value(Some)
        } else {
            Ok(Success {
                value: None,
                lexer,
            })
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// implies
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which attempts a `left` parse followed by an attempted
/// `right` parse only if `left` succeeded. If the `left` parse fails, the
/// result value is `None`, otherwise the result value is an (`Option`-wrapped)
/// tuple of the respective parsers' results.
///
/// # Similar combinators
///
/// This combinator is similar to `maybe(both(L, R))` except that `implies`
/// will return an error if the R parse fails.
pub fn implies<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<(X, Y)>>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let left_span = span!(Level::DEBUG, "ante").entered();
        let (ante, succ) = maybe(&mut left)
            (lexer, ctx.clone())?
            .take_value();

        let _ = left_span.exit();
        let _right_span = span!(Level::DEBUG, "cons").entered();
        match ante {
            None => {
                event!(Level::TRACE, "not attempted");
                Ok(succ.map_value(|_| None))
            },
            Some(l) => right
                (succ.lexer, ctx)
                .map_value(|r| Some((l, r))),
        }
    }
}

/// Returns a parser which attempts a `left` parse followed by an attempted
/// `right` parse only if `left` succeeded. If the `left` parse fails, the
/// result value is `None`, otherwise the `left` result value is returned.
///
/// # Similar combinators
///
/// This combinator is similar to `maybe(left(L, R))` except that `antecedent`
/// will return an error if the R parse fails.
pub fn antecedent<'text, Sc, L, R, X, Y>(left: L, right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<X>>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, Y>,
{
    map(
        implies(left, right),
        |v| v.map(|(l, _)| l))
}


/// Returns a parser which attempts a `left` parse followed by an attempted
/// `right` parse only if `left` succeeded. If the `left` parse fails, the
/// result value is `None`, otherwise the `right` result value is returned.
///
///
/// # Similar combinators
///
/// This combinator is similar to `maybe(right(L, R))` except that `consequent`
/// will return an error if the R parse fails.
pub fn consequent<'text, Sc, L, R, X, Y>(left: L, right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<Y>>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, Y>,
{
    map(
        implies(left, right),
        |v| v.map(|(_, r)| r))
}


////////////////////////////////////////////////////////////////////////////////
// cond_implies
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which attempts a `left` parse followed by an attempted
/// `right` parse only if `left` succeeded and its result satisfies the given
/// predicate. If the `left` parse fails, the result value is `None`, otherwise
/// the result value is a tuple of the
/// respective parsers' results.
///
/// # Similar combinators
///
/// This combinator is similar to `implies(L, R)` except that the predicate can
/// be used to force a `Some` or `None` value on the `right` parser.
///
/// This combinator is similar to `cond(..., P)` except that the predicate can
/// be passed the result of the `left` parser.
pub fn cond_implies<'text, Sc, P, L, R, X, Y>(
    mut left: L,
    mut pred: P,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<(X, Option<Y>)>>
    where
        Sc: Scanner,
        P: FnMut(&X) -> bool,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let left_span = span!(Level::DEBUG, "ante").entered();
        let (ante, succ) = maybe(&mut left)
            (lexer, ctx.clone())?
            .take_value();

        let _ = left_span.exit();
        let _right_span = span!(Level::DEBUG, "cons").entered();
        match ante {
            None => {
                event!(Level::TRACE, "not attempted");
                Ok(succ.map_value(|_| None))
            },
            Some(l) => {
                let branch = (pred)(&l);
                event!(Level::TRACE, "cond_implies: branch={}", branch);
                if branch {
                    right
                        (succ.lexer, ctx)
                        .map_value(|r| Some((l, Some(r))))
                } else {
                    Ok(Success {
                        value: Some((l, None)),
                        lexer: succ.lexer,
                    })
                }
            }   
        }
    }
}
