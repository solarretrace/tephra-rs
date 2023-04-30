////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Conditional combinators.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::maybe;
use crate::map;
use tephra::Context;
use tephra::Lexer;
use tephra::Success;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;

// External library imports.
use tephra_tracing::Level;
use tephra_tracing::span;



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
        let _span = span!(Level::DEBUG, "implies").entered();

        let (ante, succ) = maybe(&mut left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        match ante {
            None => Ok(succ.map_value(|_| None)),
            Some(l) => right
                (succ.lexer, ctx)
                .trace_result(Level::TRACE, "right capture")
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
// cond
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts a parse only if the given predicate is true.
///
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
        let _span = span!(Level::DEBUG, "cond").entered();

        if pred() {
            parser(lexer, ctx)
                .trace_result(Level::TRACE, "true branch")
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
        let _span = span!(Level::DEBUG, "implies").entered();

        let (ante, succ) = maybe(&mut left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        match ante {
            None => Ok(succ.map_value(|_| None)),
            Some(l) if (pred)(&l) => {
                right
                    (succ.lexer, ctx)
                    .trace_result(Level::TRACE, "right capture")
                    .map_value(|r| Some((l, Some(r))))
            }
            Some(l) => {
                Ok(Success {
                    value: Some((l, None)),
                    lexer: succ.lexer,
                })
            }   
        }
    }
}
