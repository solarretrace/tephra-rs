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
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which sequences two parsers wich must both succeed,
/// returning the value of the first one.
pub fn left<'text, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        R: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
{
    move |lexer| {
        let (l, succ) = (left)
            (lexer)?
            .take_value();

        (right)
            (succ.lexer)
            .map_value(|_| l)
    }
}

/// Returns a parser which sequences two parsers wich must both succeed,
/// returning the value of the second one.
pub fn right<'text, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        R: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
{
    move |lexer| {
        let succ = (left)
            (lexer)?;

        (right)
            (succ.lexer)
    }
}

/// Returns a parser which sequences two parsers wich must both succeed,
/// returning their values in a tuple.
pub fn both<'text, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, (X, Y)>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        R: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
{
    move |lexer| {
        let (l, succ) = (left)
            (lexer)?
            .take_value();

        (right)
            (succ.lexer)
            .map_value(|r| (l, r))
    }
}

/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser.
pub fn bracket<'text, Sc, Nl, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        C: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
        R: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Z>,
{
    move |lexer| {
        let succ = (left)
            (lexer)?;

        let (c, succ) = (center)
            (succ.lexer)?
            .take_value();

        (right)
            (succ.lexer)
            .map_value(|_| c)
    }
}

/// Returns a parser which calls a bracketting parser before and after a center
/// parser.
pub fn bracket_symmetric<'text, Sc, Nl, C, B, X, Y>(
    mut bracket: B,
    mut center: C)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        B: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        C: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
{
    move |lexer| {
        let succ = (&mut bracket)
            (lexer)?;

        let (c, succ) = (center)
            (succ.lexer)?
            .take_value();

        (&mut bracket)
            (succ.lexer)
            .map_value(|_| c)
    }
}

/// Returns a parser which sequences three parsers which must all succeed,
/// returning the value of the center parser. The right parser will receive the
/// output of the left parser as an argument.
pub fn bracket_dynamic<'text, Sc, Nl, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, X>,
        C: FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Y>,
        R: FnMut(Lexer<'text, Sc, Nl>, X) -> ParseResult<'text, Sc, Nl, Z>,
{
    move |lexer| {
        let (l, succ) = (left)
            (lexer)?
            .take_value();

        let (c, succ) = (center)
            (succ.lexer)?
            .take_value();

        (right)
            (succ.lexer, l)
            .map_value(|_| c)
    }
}
