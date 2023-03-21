////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for joining and bracketting.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
use crate::one;
use crate::any_index;
use crate::spanned;
use crate::recover_option;
use crate::recover_option_delayed;
use tephra::Context;
use tephra::Lexer;
use tephra::Recover;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::ParseError;
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the first one.
pub fn left<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "left").entered();

        let (l, succ) = (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        (right)
            (succ.lexer, ctx)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| l)
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the second one.
pub fn right<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "right").entered();

        let succ = (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left discard")?;

        (right)
            (succ.lexer, ctx)
            .trace_result(Level::TRACE, "right capture")
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning their values in a tuple.
pub fn both<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, (X, Y)>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "both").entered();

        let (l, succ) = (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        (right)
            (succ.lexer, ctx)
            .trace_result(Level::TRACE, "right capture")
            .map_value(|r| (l, r))
    }
}

/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser.
pub fn center<'text, Sc, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        C: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Z>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "center").entered();

        let succ = match (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left discard")
        {
            Ok(succ) => succ,
            Err(fail) => return Err(fail),
        };

        let (c, succ) = match (center)
            (succ.lexer, ctx.clone())
            .trace_result(Level::TRACE, "center capture")
        {
            Ok(succ) => succ.take_value(),
            Err(fail) => return Err(fail),
        };

        (right)
            (succ.lexer, ctx)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| c)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Bracket combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which brackets the given parser in a pair of tokens.
///
/// Attempts error recovery if the given parser fails by scanning for the right
/// token. If the right token is not found, an unmatched delimiter error will
/// be emitted.
pub fn bracket<'text, Sc, F, X>(
    left_token: Sc::Token,
    center: F,
    right_token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<X>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
{
    let mut recover_center = recover_option(
        center,
        Recover::before(right_token.clone()));

    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "bracket").entered();

        let (l, succ) = spanned(one(left_token.clone()))
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left_token discard")?
            .take_value();

        let ctx = ctx
            .push(std::rc::Rc::new(move |e| if e.is_recover_error() {
                ParseError::unmatched_delimiter(
                    *e.source_text(),
                    "unmatched delimiter",
                    l.span.clone())
            } else {
                e
            }));

        let (c, succ) = match (recover_center)
            (succ.lexer, ctx.clone())
            .trace_result(Level::TRACE, "center capture")
            .apply_context(ctx.clone())
        {
            Ok(succ) => succ.take_value(),
            Err(fail) => return Err(fail),
        };

        one(right_token.clone())
            (succ.lexer, ctx.clone())
            .trace_result(Level::TRACE, "right_token discard")
            .map_value(|_| c)
    }
}

/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens.
/// 
/// Each token in the `left_tokens` slice is paired with the token of the same
/// index in the `right_tokens` slice. Each slice must be the same length.
///
/// Attempts error recovery if the given parser fails by scanning for the right
/// token. If the right token is not found, an unmatched delimiter error will
/// be emitted.
pub fn bracket_matching<'text: 'a, 'a, Sc, F, X: 'a>(
    left_tokens: &'a [Sc::Token],
    center: F,
    right_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>)
        -> ParseResult<'text, Sc, Option<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text>)
            -> ParseResult<'text, Sc, X> + 'a,
{
    if left_tokens.len() != right_tokens.len() || left_tokens.is_empty() {
        panic!("invalid argument to bracket_matching");
    }

    let mut recover_center =  recover_option_delayed(center);

    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "bracket_matching").entered();

        let (l, succ) = spanned(any_index(left_tokens))
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left_token discard")?
            .take_value();
        let idx = l.value;
        let span = l.span;
        let recover = Recover::before(right_tokens[idx].clone());

        let ctx = ctx
            .push(std::rc::Rc::new(move |e| if e.is_recover_error() {
                ParseError::unmatched_delimiter(
                    *e.source_text(),
                    "unmatched delimiter",
                    span.clone())
            } else {
                e
            }));

        let (c, succ) = match (recover_center)
            (succ.lexer, ctx.clone(), recover)
            .trace_result(Level::TRACE, "center capture")
            .apply_context(ctx.clone())
        {
            Ok(succ) => succ.take_value(),
            Err(fail) => return Err(fail),
        };

        one(right_tokens[idx].clone())
            (succ.lexer, ctx.clone())
            .trace_result(Level::TRACE, "right_token discard")
            .map_value(|_| c)
    }
}


