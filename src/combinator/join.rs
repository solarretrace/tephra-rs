////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for joining and bracketting.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;

// External library imports.
use tracing::Level;
use tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers which must both succeed,
/// returning the value of the first one.
pub fn left<'text, Sc, Cm, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        L: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        R: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "left");
        let _enter = span.enter();

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
pub fn right<'text, Sc, Cm, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        L: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        R: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "right");
        let _enter = span.enter();

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
pub fn both<'text, Sc, Cm, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, (X, Y)>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        L: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        R: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "both");
        let _enter = span.enter();

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
pub fn bracket<'text, Sc, Cm, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        L: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        C: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
        R: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Z>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "bracket");
        let _enter = span.enter();

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

/// Returns a parser which calls a bracketting parser before and after a center
/// parser.
pub fn bracket_symmetric<'text, Sc, Cm, C, B, X, Y>(
    mut bracket: B,
    mut center: C)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        B: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        C: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "bracket_symmetric");
        let _enter = span.enter();

        let succ = (&mut bracket)
            (lexer)
            .trace_result(Level::TRACE, "left discard")?;

        let (c, succ) = (center)
            (succ.lexer)
            .trace_result(Level::TRACE, "center capture")?
            .take_value();

        (&mut bracket)
            (succ.lexer)
            .trace_result(Level::TRACE, "right discard")
            .map_value(|_| c)
    }
}

/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser. The right parser will receive the
/// output of the left parser as an argument.
pub fn bracket_dynamic<'text, Sc, Cm, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
        L: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, X>,
        C: FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Y>,
        R: FnMut(Lexer<'text, Sc, Cm>, X) -> ParseResult<'text, Sc, Cm, Z>,
{
    move |lexer| {
        let span = span!(Level::DEBUG, "bracket_dynamic");
        let _enter = span.enter();

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
