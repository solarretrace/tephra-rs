////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for repeating.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::discard;
use crate::empty;
use crate::one;
use crate::right;

// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;


////////////////////////////////////////////////////////////////////////////////
// Repetition combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which repeats the given number of times. The parsed value
/// is the number of successful parses.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn repeat_count<'text, Sc, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        intersperse_count(low, high, 
                discard(&mut parser),
                empty)
            (lexer, ctx)
    }
}

/// Returns a parser which repeats the given number of times. The parsed value
/// is the number of successful parses.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn repeat_count_until<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
{
    move |lexer, ctx| {
        intersperse_count_until(low, high, 
                &mut stop_parser,
                discard(&mut parser),
                empty)
            (lexer, ctx)
    }
}

/// Returns a parser which repeats the given number of times. Each parsed value
/// is collected into a `Vec`.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn repeat<'text, Sc, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer, ctx| {
        intersperse(low, high,
            &mut parser,
            empty)
            (lexer, ctx)
    }
}

/// Returns a parser which repeats the given number of times or until a stop
/// parser succeeds, interspersed by parse attempts from a secondary parser.
/// Each parsed value is collected into a `Vec`. The stop parse is not included
/// in the result.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn repeat_until<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<U>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
{
    move |lexer, ctx| {
        intersperse_until(low, high, &mut stop_parser,
            &mut parser,
            empty)
            (lexer, ctx)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Intersperse combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn intersperse_count<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
{
    move |lexer, ctx| {
        intersperse(low, high, 
                discard(&mut parser),
                &mut inter_parser)
            (lexer, ctx)
            .map_value(|vals| vals.len())
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn intersperse_count_until<'text, Sc, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
        H: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, T>,
{
    move |lexer, ctx| {
        intersperse_until(low, high, 
                &mut stop_parser,
                discard(&mut parser),
                &mut inter_parser)
            (lexer, ctx)
            .map_value(|vals| vals.len())
    }
}

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. Each parsed value is collected into
/// a `Vec`.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn intersperse<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
{
    move |lexer, ctx| {
        if let Some(h) = high {
            assert!(h >= low, "intersperse with high < low");
            if h == 0 {
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        let (val, mut succ) = match parser
            (lexer.clone(), ctx.clone())
        {
            Ok(Success { value, lexer: succ_lexer }) => {
                (value, Success { value: (), lexer: succ_lexer })
            },
            Err(fail) => return if low == 0 {
                Ok(Success { lexer, value: vals })
            } else {
                Err(fail)
            },
        };

        vals.push(val);

        while vals.len() < low {
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer, ctx.clone())?
                .take_value();
            vals.push(val);
            succ = next;
        }

        while high.map_or(true, |h| vals.len() < h) {
            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone(), ctx.clone())
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
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn intersperse_until<'text, Sc, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<U>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, U>,
        H: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, T>,
{
    move |lexer, ctx| {
        if let Some(h) = high {
            assert!(h >= low, "intersperse with high < low");
            if h == 0 {
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        if stop_parser
            (lexer.clone(), ctx.clone())
            .is_ok()
        {
            return Ok(Success { lexer, value: vals });
        }

        let (val, mut succ) = parser
            (lexer, ctx.clone())?
            .take_value();
        vals.push(val);

        while vals.len() < low {
            if stop_parser
                (succ.lexer.clone(), ctx.clone())
                .is_ok()
            {
                return Ok(succ.map_value(|_| vals));
            }
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer, ctx.clone())?
                .take_value();
            vals.push(val);
            succ = next;
        }

        while high.map_or(true, |h| vals.len() < h) {
            if stop_parser
                (succ.lexer.clone(), ctx.clone())
                .is_ok()
            {
                return Ok(succ.map_value(|_| vals));
            }

            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone(), ctx.clone())
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
/// parse attempts from a secondary parser. Each parsed value is collected into
/// a `Vec`.
///
/// # Panics
///
/// Panics if `high` < `low`.
pub fn intersperse_default<'text, Sc, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    sep_token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, V>,
        V: Default,
{
    move |lexer, ctx| {
        if let Some(h) = high {
            assert!(h >= low, "invalid argument to intersperse_default: high < low");
            if h == 0 {
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        let (val, mut succ) = match parser
            (lexer.clone(), ctx.clone())
        {
            Ok(Success { value, lexer: succ_lexer }) => {
                (value, Success { value: (), lexer: succ_lexer })
            },
            Err(_) if low == 0 => {
                return Ok(Success { lexer, value: vals });
            },
            Err(fail) =>  {
                return Err(fail);
            },
        };

        vals.push(val);

        while vals.len() < low {
            let (val, next) = right(one(sep_token.clone()), &mut parser)
                (succ.lexer, ctx.clone())?
                .take_value();
            vals.push(val);
            succ = next;
        }

        while high.map_or(true, |h| vals.len() < h) {
            match right(one(sep_token.clone()), &mut parser)
                (succ.lexer.clone(), ctx.clone())
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
