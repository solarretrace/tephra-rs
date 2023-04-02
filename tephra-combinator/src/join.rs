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
use crate::one;
use crate::spanned;
use crate::recover_option;

// External library imports.
use tephra::Context;
use tephra::Lexer;
use tephra::Failure;
use tephra::Success;
use tephra::Recover;
use tephra::SpanDisplay;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::ParseError;
use tephra_tracing::Level;
use tephra_tracing::span;
use smallvec::SmallVec;


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
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>
    where
        Sc: Scanner,
        L: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, X>,
        R: FnMut(Lexer<'text, Sc>, Context<'text, Sc>) -> ParseResult<'text, Sc, Y>,
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

/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens.
/// 
/// Each token in `open_tokens` is paired with the token of the same index in
/// `close_tokens`. Each slice must be the same length, and must not contain any
/// shared tokens.
///
/// The `abort_tokens` argument can be used to limit the search for open
/// brackets by failing if any of the abort tokens is encountered before an open
/// token. If a valid open token is guaranteed to be found, this can be empty.
///
/// ## Error recovery
///
/// Attempts error recovery if the given parser fails by scanning for the right
/// token. If the right token is not found, an unmatched delimiter error will
/// be emitted.
pub fn bracket_matching<'text: 'a, 'a, Sc, F, X: 'a>(
    open_tokens: &'a [Sc::Token],
    mut center: F,
    close_tokens: &'a [Sc::Token],
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, (Option<X>, usize)> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
{ 
    if open_tokens.is_empty() || close_tokens.is_empty() {
        panic!("invalid argument to bracket_matching: empty token slice");
    }
    if open_tokens.len() != close_tokens.len() || open_tokens.is_empty() {
        panic!("invalid argument to bracket_matching: mismatched token \
            slice lengths");
    }
    for tok in open_tokens {
        if close_tokens.contains(tok) {
            panic!("invalid argument to bracket_matching: common open and \
                close tokens unsupported");       
        }
    }

    use BracketError::*;

    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "bracket_matching").entered();

        match match_nested_brackets(lexer.clone(),
            open_tokens,
            close_tokens,
            abort_tokens)
        {
            Err(NoneFound(lexer)) => {
                // TODO: Better error message.
                Err(Failure {
                    parse_error: ParseError::new(
                        lexer.source_text(),
                        "Expected [bracket]"),
                    lexer,
                })
            },
            Err(Unopened(lexer)) => {
                // TODO: Better error message.
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "unmatched close bracket"),
                    lexer,
                })
            },
            Err(Unclosed(mut lexer)) => {
                // TODO: Better error message.
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "unmatched open bracket")
                        .with_span_display(SpanDisplay::new_error_highlight(
                            lexer.source_text(),
                            lexer.peek_token_span().unwrap(),
                            "this bracket is not closed",
                        )),
                    lexer,
                })
            },
            Err(Mismatch(_open_lexer, _close_lexer)) => {
                // TODO: Better error message.
                Err(Failure {
                    parse_error: ParseError::new(
                        lexer.source_text(),
                        "open bracket found with wrong close"),
                    lexer,
                })
            },

            Ok(BracketMatch {mut open, mut close, index }) => {
                // Prepare the sublexers.
                let _ = open.next();
                // println!("open: {}", open);
                let center_lexer = open.sublexer();
                let _ = close.next();
                // println!("center_lexer: {}", center_lexer.source_text());
                // println!("close: {}", close);

                // NOTE: We will do manual error recovery here: we don't want to
                // advance to a token, because they're potentially nested. We
                // instead substitute the close, which is already pointing
                // to the proper recovery position.
                match (center)
                    (center_lexer, ctx.clone())
                    .map_value(|v| (Some(v), index))
                {
                    Ok(succ) => Ok(Success {
                        value: succ.value,
                        lexer: close,
                    }),
                    Err(fail) => {
                        println!("{fail}");
                        match ctx.send_error(fail.parse_error, &fail.lexer)
                        {
                            Err(parse_error) => Err(Failure {
                                parse_error,
                                lexer: close,
                            }),
                            Ok(()) => Ok(Success {
                                value: (None, index),
                                lexer: close,
                            }),
                        }
                    },
                }
            },
        }
    }
}

/// Returns a pair of lexers such that the first lexer's `next` token is in
/// `open_tokens` and the second lexer's `next` token is the corresponding entry
/// in `right_tokens`.
///
/// Each token in `open_tokens` is paired with the token of the same index in
/// `close_tokens`. Each slice must be the same length, and must not contain any
/// shared tokens.
///
/// The `abort_tokens` argument can be used to limit the search for open
/// brackets by failing if any of the abort tokens is encountered before an open
/// token. If a valid open token is guaranteed to be found, this can be empty.
fn match_nested_brackets<'text: 'a, 'a, Sc>(
    mut lexer: Lexer<'text, Sc>,
    open_tokens: &'a [Sc::Token],
    close_tokens: &'a [Sc::Token],
    abort_tokens: &'a [Sc::Token])
    -> Result<BracketMatch<'text, Sc>, BracketError<'text, Sc>>
    where Sc: Scanner
{
    use BracketError::*;
    let mut open_lexer = None;
    // Detected open tokens as (index, count) pairs.
    let mut opened: SmallVec<[(usize, usize); DEFAULT_TOKEN_VEC_SIZE]>
        = SmallVec::with_capacity(open_tokens.len());

    while let Some(tok) = lexer.peek() {
        if let Some(idx) = close_tokens.iter().position(|t| t == &tok) {
            match opened.pop() {
                // Close token found before open token.
                None => return Err(Unopened(lexer)),

                // Wrong close token for current open token.
                Some((t, _)) if t != idx 
                    => return Err(Mismatch(open_lexer.unwrap(), lexer)),

                Some((t, n)) if t == idx && n > 1 => {
                    opened.push((t, n-1));
                },
                Some((t, n)) if t == idx && n == 1 && opened.is_empty() => {
                    return Ok(BracketMatch {
                        open: open_lexer.unwrap(),
                        close: lexer,
                        index: idx,
                    });
                },
                Some(_) => unreachable!(),
            }
        } else if let Some(idx) = open_tokens.iter().position(|t| t == &tok) {
            // Found our first open token.
            if open_lexer.is_none() { 
                open_lexer = Some(lexer.clone());
            }
            match opened.pop() {
                None => {
                    opened.push((idx, 1));
                },
                Some((t, n)) if t != idx => {
                    opened.push((t, n));
                    opened.push((idx, 1));
                },
                Some((t, n)) if t == idx => {
                    opened.push((t, n+1));
                },
                Some(_) => unreachable!(),
            }
        } else if abort_tokens.contains(&tok) && open_lexer.is_none() {
            // Abort search if no open tokens found before abort token.
            return Err(NoneFound(lexer));
        }

        let _ = lexer.next();
    }

    match open_lexer {
        // End-of-text reached.
        None    => Err(NoneFound(lexer)),
        // Unclosed open token.
        Some(o) => Err(Unclosed(o)),
    }
}


const DEFAULT_TOKEN_VEC_SIZE: usize = 4;

struct BracketMatch<'text, Sc> where Sc: Scanner {
    open: Lexer<'text, Sc>,
    close: Lexer<'text, Sc>,
    index: usize,
}

enum BracketError<'text, Sc> where Sc: Scanner {
    NoneFound(Lexer<'text, Sc>),
    Unopened(Lexer<'text, Sc>),
    Unclosed(Lexer<'text, Sc>),
    Mismatch(Lexer<'text, Sc>, Lexer<'text, Sc>),
}
