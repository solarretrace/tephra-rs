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
use tephra::lexer::Lexer;
use tephra::lexer::Scanner;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;

// External library imports.
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the first one.
pub fn left<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "left").entered();

        let (l, succ) = (left)
            (lexer)
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        (right)
            (succ.lexer)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| l)
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the second one.
pub fn right<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "right").entered();

        let succ = (left)
            (lexer)
            .trace_result(Level::TRACE, "left discard")?;

        (right)
            (succ.lexer)
            .trace_result(Level::TRACE, "right capture")
    }
}

/// Returns a parser which sequences two parsers which must both succeed,
/// returning their values in a tuple.
pub fn both<'text, Sc, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, (X, Y)>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "both").entered();

        let (l, succ) = (left)
            (lexer)
            .trace_result(Level::TRACE, "left capture")?
            .take_value();

        (right)
            (succ.lexer)
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
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>,
        C: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>,
        R: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Z>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "bracket").entered();

        let succ = match (left)
            (lexer)
            .trace_result(Level::TRACE, "left discard")
        {
            Ok(succ) => succ,
            Err(fail) => return Err(fail),
        };

        let (c, succ) = match (center)
            (succ.lexer)
            .trace_result(Level::TRACE, "center capture")
        {
            Ok(succ) => succ.take_value(),
            Err(fail) => return Err(fail),
        };

        (right)
            (succ.lexer)
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
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, X>,
        C: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Y>,
        R: FnMut(Lexer<'text, Sc>, X) -> ParseResult<'text, Sc, Z>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "bracket_dynamic").entered();

        let (l, succ) = (left)
            (lexer)
            .trace_result(Level::TRACE, "left discard")?
            .take_value();

        let (c, succ) = (center)
            (succ.lexer)
            .trace_result(Level::TRACE, "center capture")?
            .take_value();

        (right)
            (succ.lexer, l)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| c)
    }
}
