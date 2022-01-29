////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for repeating.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::discard;
use crate::empty;
use crate::right;
use tephra::lexer::Lexer;
use tephra::lexer::Scanner;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tephra::result::Success;

// External library imports.
use tracing::event;
use tracing::Level;
use tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Repetition combinators.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. The parsed value is the number of
/// successful parses.
pub fn repeat<'text, Sc, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
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
pub fn repeat_until<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
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

/// Returns a parser which repeats the given number of times. Each parsed value
/// is collected into a `Vec`.
pub fn repeat_collect<'text, Sc, F, V>(
    low: usize,
    high: Option<usize>,
    mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Vec<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
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
pub fn repeat_collect_until<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Vec<U>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
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
pub fn intersperse<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
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
pub fn intersperse_until<'text, Sc, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, usize>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
        H: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, T>,
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

/// Returns a parser which repeats the given number of times, interspersed by
/// parse attempts from a secondary parser. Each parsed value is collected into
/// a `Vec`.
pub fn intersperse_collect<'text, Sc, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inter_parser: G)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Vec<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "intersperse_collect").entered();

        event!(Level::DEBUG, "low = {:?}, high = {:?}", low, high);

        if let Some(h) = high {
            if h < low { panic!("intersperse_collect with high < low") }
            if h == 0 {
                event!(Level::TRACE, "Ok with 0 repetitions");
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        let (val, mut succ) = match (&mut parser)
            (lexer.clone())
            .trace_result(Level::TRACE, "first subparse")
        {
            Ok(Success { value, lexer: succ_lexer }) => {
                (value, Success { value: (), lexer: succ_lexer })
            },
            Err(fail) => return if low == 0 {
                event!(Level::TRACE, "Ok with 0 repetitions");
                Ok(Success { lexer, value: vals })
            } else {
                event!(Level::TRACE, "Err with 0 repetitions");
                Err(fail)
            },
        };

        vals.push(val);
        event!(Level::TRACE, "1 repetition...");

        while vals.len() < low {
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer)
                .trace_result(Level::TRACE, "continuing subparse")?
                .take_value();
            vals.push(val);
            event!(Level::TRACE, "{} repetitions...", vals.len());
            succ = next;
        }

        event!(Level::TRACE, "minimum count satisfied");

        while high.map_or(true, |h| vals.len() < h) {
            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone())
                .trace_result(Level::TRACE, "continuing subparse")
            {
                Ok(next) => {
                    let (val, next) = next.take_value();
                    vals.push(val);
                    event!(Level::TRACE, "{} repetitions...", vals.len());
                    succ = next;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| vals.len() >= h) {
                event!(Level::TRACE, "maximum count satisfied");
                break;
            }
        }

        event!(Level::TRACE, "Ok with {} repetitions", vals.len());
        Ok(succ.map_value(|_| vals))
    }
}

/// Returns a parser which repeats the given number of times or until a stop
/// parser succeeds, interspersed by parse attempts from a secondary parser.
/// Each parsed value is collected into a `Vec`. The stop parse is not included
/// in the result.
pub fn intersperse_collect_until<'text, Sc, F, G, H, V, U, T>(
    low: usize,
    high: Option<usize>,
    mut stop_parser: F,
    mut parser: G,
    mut inter_parser: H)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Vec<U>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
        G: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, U>,
        H: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, T>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "intersperse_collect_until").entered();

        event!(Level::DEBUG, "low = {:?}, high = {:?}", low, high);

        if let Some(h) = high {
            if h < low { panic!("intersperse_collect with high < low") }
            if h == 0 {
                event!(Level::TRACE, "Ok with 0 repetitions");
                return Ok(Success {
                    lexer,
                    value: Vec::new(),
                });
            }
        }

        let mut vals = Vec::with_capacity(4);

        if (&mut stop_parser)
            (lexer.clone())
            .trace_result(Level::TRACE, "stop subparse")
            .is_ok()
        {
            return Ok(Success { lexer, value: vals });
        }

        let (val, mut succ) = (&mut parser)
            (lexer)
            .trace_result(Level::TRACE, "first subparse")?
            .take_value();
        vals.push(val);
        event!(Level::TRACE, "1 repetition...");

        while vals.len() < low {
            if (&mut stop_parser)
                (succ.lexer.clone())
                .trace_result(Level::TRACE, "stop subparse")
                .is_ok()
            {
                return Ok(succ.map_value(|_| vals));
            }
            let (val, next) = right(&mut inter_parser, &mut parser)
                (succ.lexer)
                .trace_result(Level::TRACE, "continuing subparse")?
                .take_value();
            vals.push(val);
            event!(Level::TRACE, "{} repetitions...", vals.len());
            succ = next;
        }

        event!(Level::TRACE, "minimum count satisfied");

        while high.map_or(true, |h| vals.len() < h) {
            if (&mut stop_parser)
                (succ.lexer.clone())
                .trace_result(Level::TRACE, "stop subparse")
                .is_ok()
            {
                return Ok(succ.map_value(|_| vals));
            }

            match right(&mut inter_parser, &mut parser)
                (succ.lexer.clone())
                .trace_result(Level::TRACE, "continuing subparse")
            {
                Ok(next) => {
                    let (val, next) = next.take_value();
                    vals.push(val);
                    event!(Level::TRACE, "{} repetitions...", vals.len());
                    succ = next;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| vals.len() >= h) {
                event!(Level::TRACE, "maximum count satisfied");
                break;
            }
        }

        event!(Level::TRACE, "Ok with {} repetitions", vals.len());
        Ok(succ.map_value(|_| vals))
    }
}
