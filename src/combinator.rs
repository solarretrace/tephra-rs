////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::Success;
use crate::primitive::empty;


////////////////////////////////////////////////////////////////////////////////
// Control combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which disables all token filters during exectution of the given
/// parser.
pub fn exact<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |mut lexer| {
        let filter = lexer.take_filter();
        match (parser)(lexer) {
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
// Parse result substitution combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer) {
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
pub fn text<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, &'t str>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        let start = lexer.current_pos().byte;
        match (parser)(lexer) {
            Ok(succ) => {
                let end = succ.lexer.current_pos().byte;
                let value = &succ.lexer.source()[start..end];

                Ok(Success {
                    lexer: succ.lexer,
                    value,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse result selection combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which sequences two parsers wich must both succeed,
/// returning the value of the first one.
pub fn left<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
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
pub fn right<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
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
pub fn both<'t, Sc, Nl, L, R, X, Y>(mut left: L, mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, (X, Y)>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
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


/// Returns a parser which sequences two parsers wich must both succeed,
/// returning their values in a tuple.
pub fn bracket<'t, Sc, Nl, L, C, R, X, Y, Z>(
    mut left: L,
    mut center: C,
    mut right: R)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>
    where
        Sc: Scanner,
        Nl: NewLine,
        L: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, X>,
        C: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Y>,
        R: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Z>,
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

// bracket_with
// circumfix

////////////////////////////////////////////////////////////////////////////////
// Tolerance & inversion combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Option<V>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer.clone()) {
            Ok(succ) => Ok(succ.map_value(Some)),
            Err(_fail) => Ok(Success {
                    lexer,
                    value: None,
            }),
        }

    }
}

// require_if


////////////////////////////////////////////////////////////////////////////////
// Repetition combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. Each parsed value is collected into
/// a `Vec`.
pub fn intersperse_collect<'t, Sc, Nl, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Vec<V>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
{
    move |lexer| {
        if let Some(h) = high {
            if h < low { panic!("intersperse_collect with high < low") }
            if h == 0 {
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        let (val, mut succ) = (&mut parser)
            (lexer)?
            .take_value();
        vals.push(val);

        while vals.len() < low {
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer)?
                .take_value();
            vals.push(val);
            succ = next;
        }

        while high.map_or(true, |h| vals.len() < h) {
            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone())
            {
                Ok(next) => {
                    let (val, next) = next.take_value();
                    vals.push(val);
                    succ = next;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| vals.len() >= h) {
                break;
            }
        }

        Ok(succ.map_value(|_| vals))
    }
}

/// Returns a parser which repeats the given number of times or until a stop
/// parser succeeds, interspersed by parse attempts from a secondary parser.
/// Each parsed value is collected into a `Vec`. The stop parse is not included
/// in the result.
pub fn intersperse_collect_until<'t, Sc, Nl, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Vec<U>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
        H: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, T>,
{
    move |lexer| {
        if let Some(h) = high {
            if h < low { panic!("intersperse_collect with high < low") }
            if h == 0 {
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        if (&mut stop_parser)(lexer.clone()).is_ok() {
            return Ok(Success { lexer, value: vals });
        }

        let (val, mut succ) = (&mut parser)
            (lexer)?
            .take_value();
        vals.push(val);

        while vals.len() < low {
            if (&mut stop_parser)(succ.lexer.clone()).is_ok() {
                return Ok(succ.map_value(|_| vals));
            }
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer)?
                .take_value();
            vals.push(val);
            succ = next;
        }

        while high.map_or(true, |h| vals.len() < h) {
            if (&mut stop_parser)(succ.lexer.clone()).is_ok() {
                return Ok(succ.map_value(|_| vals));
            }

            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone())
            {
                Ok(next) => {
                    let (val, next) = next.take_value();
                    vals.push(val);
                    succ = next;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| vals.len() >= h) {
                break;
            }
        }

        Ok(succ.map_value(|_| vals))
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
pub fn intersperse<'t, Sc, Nl, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, usize>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
{
    move |lexer| {
        intersperse_collect(low, high, 
                discard(&mut parser),
                &mut inter_parser)
            (lexer)
            .map_value(|vals| vals.len())
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
pub fn intersperse_until<'t, Sc, Nl, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, usize>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
        H: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, T>,
{
    move |lexer| {
        intersperse_collect_until(low, high, 
                &mut stop_parser,
                discard(&mut parser),
                &mut inter_parser)
            (lexer)
            .map_value(|vals| vals.len())
    }
}

/// Returns a parser which repeats the given number of times. Each parsed value
/// is collected into a `Vec`.
pub fn repeat_collect<'t, Sc, Nl, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Vec<V>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        intersperse_collect(low, high,
            &mut parser,
            empty)
            (lexer)
    }
}

/// Returns a parser which repeats the given number of times or until a stop
/// parser succeeds, interspersed by parse attempts from a secondary parser.
/// Each parsed value is collected into a `Vec`. The stop parse is not included
/// in the result.
pub fn repeat_collect_until<'t, Sc, Nl, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, Vec<U>>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
{
    move |lexer| {
        intersperse_collect_until(low, high, &mut stop_parser,
            &mut parser,
            empty)
            (lexer)
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
pub fn repeat<'t, Sc, Nl, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, usize>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        intersperse_collect(low, high, 
                discard(&mut parser),
                empty)
            (lexer)
            .map_value(|vals| vals.len())
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
pub fn repeat_until<'t, Sc, Nl, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, usize>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
        G: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, U>,
{
    move |lexer| {
        intersperse_collect_until(low, high, 
                &mut stop_parser,
                discard(&mut parser),
                empty)
            (lexer)
            .map_value(|vals| vals.len())
    }
}
