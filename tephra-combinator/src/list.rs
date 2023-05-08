////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for delimitted brackets and lists.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::empty;
use crate::discard;
use crate::one;
use crate::recover_default;
use crate::stabilize;

// External library imports.
use tephra::error::ParseBoundaryError;
use tephra::error::RepeatCountError;
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;

////////////////////////////////////////////////////////////////////////////////
// Miscellaneous delimiter combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which requires the given parser to produce a value by
/// consuming up to one of the given tokens.
///
/// If the given parser is successful, but doesn't terminate on a token
/// satisfying `abort_pred`, then an error is returned.
///
/// ## Error recovery
///
/// No error recovery is attempted.
pub fn up_to<'text: 'a, 'a, Sc, F, X: 'a, A>(
    mut parser: F,
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, X> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{
    move |lexer, ctx| { 
        let mut succ = (parser)(lexer, ctx)?;

        match succ.lexer.peek() {
            None                            => Ok(succ),
            Some(tok) if (abort_pred)(&tok) => Ok(succ),
            _ => {
                let parse_span = succ.lexer.parse_span();
                // Advance lexer to the expected token.
                let _ = succ.lexer.advance_to(&abort_pred);

                Err(Box::new(ParseBoundaryError {
                    parse_span,
                    expected_end_pos: succ.lexer.cursor_pos(),
                }))
            },
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// List combinators.
////////////////////////////////////////////////////////////////////////////////

pub fn delimited_list<'text: 'a, 'a, Sc, F, X: 'a, A>(
    parser: F,
    sep_token: Sc::Token,
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<Option<X>>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        A: Fn(&Sc::Token) -> bool + 'static + Clone,
{
    delimited_list_bounded(0, None, parser, sep_token, abort_pred)
}


pub fn delimited_list_bounded<'text: 'a, 'a, Sc, F, X: 'a, A>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    sep_token: Sc::Token,
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<Option<X>>> + 'a
    where
        Sc: Scanner + 'a,
        A: Fn(&Sc::Token) -> bool + 'static + Clone,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
{
    let option_parser = move |lexer, ctx| {
        (parser)
            (lexer, ctx)
            .map_value(Some)
    };

    delimited_list_bounded_default(
        low,
        high,
        option_parser,
        sep_token,
        abort_pred)
}


pub fn delimited_list_default<'text: 'a, 'a, Sc, F, X: 'a, A>(
    parser: F,
    sep_token: Sc::Token,
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
        A: Fn(&Sc::Token) -> bool + 'static + Clone,
{
    delimited_list_bounded_default(0, None, parser, sep_token, abort_pred)
}


pub fn delimited_list_bounded_default<'text: 'a, 'a, Sc, F, X: 'a, A>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    sep_token: Sc::Token,
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
        A: Fn(&Sc::Token) -> bool + 'static + Clone,
{
    let move_token = sep_token.clone();
    let move_pred = abort_pred.clone();
    let sep_or_abort_pred = move |tok: &Sc::Token| {
        tok == &move_token || (move_pred)(tok)
    };

    let rec_token = sep_token.clone();
    let rec_pred = abort_pred.clone();
    let recover_pat = Rc::new(RwLock::new(move |tok: Sc::Token| {
        Ok(tok == rec_token || rec_pred(&tok))
    }));

    move |mut lexer, ctx| {
        let mut vals = match high {
            // Do empty parse if requested.
            Some(0) => {
                return empty(lexer, ctx).map_value(|_| Vec::new());
            },
            Some(n) if n < low => {
                panic!("delimited_list with high < low");
            },
            Some(n) if n < LIST_BOUND_PRELLOCATE_LIMIT => Vec::with_capacity(n),
            Some(_) => Vec::with_capacity(LIST_BOUND_PRELLOCATE_LIMIT),
            None    => Vec::with_capacity(LIST_UNBOUND_PREALLOCATE),
        };

        let mut aborting = false;
        
        loop {
            // End loop if no remaining text.
            match lexer.peek() {
                None => { break; }
                Some(tok) if (abort_pred)(&tok) => {
                    if vals.is_empty() { break; }
                    aborting = true;
                }
                _ => (),
            }
            if lexer.is_empty() { break; }

            // Try to parse value.
            let res = stabilize(recover_default(
                    up_to(&mut parser, &sep_or_abort_pred),
                    recover_pat.clone()))
                (lexer.clone(), ctx.clone());

            if aborting {
                vals.push(X::default());
                break;
            }
            let (val, succ) = res?
                .take_value();
            lexer = succ.lexer;
            vals.push(val);

            // End loop if high is reached.
            if let Some(n) = high { if vals.len() >= n { break; } }
            // End loop if no remaining text.
            match lexer.peek() {
                None => { break; }
                Some(tok) if (abort_pred)(&tok) => { break; }
                _ => (),
            }
            if lexer.is_empty() { break; }

            // Try to parse sep token.
            let (_, succ) = stabilize(recover_default(
                    discard(one(sep_token.clone())),
                    recover_pat.clone()))
                (lexer.clone(), ctx.clone())?
                .take_value();
            lexer = succ.lexer.into_sublexer();
        }

        if vals.len() < low {
            let parse_error = Box::new(RepeatCountError {
                parse_span: lexer.parse_span(),
                found: vals.len(),
                expected_min: low,
                expected_max: high,
            });
                
            match ctx.send_error(parse_error) {
                Err(parse_error) => Err(parse_error),
                Ok(()) => Ok(Success {
                    value: vals,
                    lexer,
                }),
            }
        } else {
            Ok(Success {
                value: vals,
                lexer,
            })
        }
    }
}

const LIST_BOUND_PRELLOCATE_LIMIT: usize = 16;
const LIST_UNBOUND_PREALLOCATE: usize = 4;
