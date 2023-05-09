////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for joining and bracketting.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::map;

// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the first one.
///
/// ## Error recovery
///
/// No error recovery is attempted.
pub fn left<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        map(both(&mut left, &mut right), |(l, _)| l)
            (lexer, ctx)
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the second one.
pub fn right<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        map(both(&mut left, &mut right), |(_, r)| r)
            (lexer, ctx)
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning their values in a tuple.
///
/// ## Error recovery
///
/// No error recovery is attempted.
pub fn both<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, (X, Y)>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer, ctx| {
        let left_span = span!(Level::DEBUG, "left").entered();
        let (l, succ) = (left)
            (lexer, ctx.clone())?
            .take_value();

        let _ = left_span.exit();
        let _right_span = span!(Level::DEBUG, "right").entered();
        (right)
            (succ.lexer, ctx)
            .map_value(|r| (l, r))
    }
}

/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser.
///
/// ## Error recovery
///
/// No error recovery is attempted.
pub fn center<'text, Sc, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        C: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Z>,
{
    move |lexer, ctx| {
        let left_span = span!(Level::DEBUG, "left").entered();
        let succ = match (left)
            (lexer, ctx.clone())
        {
            Ok(succ) => succ,
            Err(fail) => return Err(fail),
        };

        let _ = left_span.exit();
        let center_span = span!(Level::DEBUG, "center").entered();
        let (c, succ) = match (center)
            (succ.lexer, ctx.clone())
        {
            Ok(succ) => succ.take_value(),
            Err(fail) => return Err(fail),
        };

        let _ = center_span.exit();
        let _right_span = span!(Level::DEBUG, "right").entered();
        (right)
            (succ.lexer, ctx)
            .map_value(|_| c)
    }
}

