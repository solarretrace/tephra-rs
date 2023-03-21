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
use crate::spanned;
use crate::recover_option;
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
pub fn bracket<'text, Sc, L, C, R, X, Y, Z>(
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
        let _span = span!(Level::DEBUG, "bracket").entered();

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


/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser. The right parser will receive the
/// output of the left parser as an argument.
pub fn bracket_dynamic<'text, Sc, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, X>,
        C: FnMut(Lexer<'text, Sc>, Context<'text>) -> ParseResult<'text, Sc, Y>,
        R: FnMut(Lexer<'text, Sc>, Context<'text>, X)
            -> ParseResult<'text, Sc, Z>,
{
    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "bracket_dynamic").entered();

        let (l, succ) = (left)
            (lexer, ctx.clone())
            .trace_result(Level::TRACE, "left discard")?
            .take_value();

        let (c, succ) = (center)
            (succ.lexer, ctx.clone())
            .trace_result(Level::TRACE, "center capture")?
            .take_value();

        (right)
            (succ.lexer, ctx, l)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| c)
    }
}


/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser.
pub fn token_bracket<'text, Sc, F, X>(
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
            .trace_result(Level::TRACE, "left discard")?
            .take_value();

        let ctx = ctx
            .push(std::rc::Rc::new(move |e| if e.is_recover_error() {
                println!("{l:?}");
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
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| c)
    }
}
