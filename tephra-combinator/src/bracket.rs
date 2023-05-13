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
use crate::map;

// External library imports.
use smallvec::SmallVec;
use tephra::Context;
use tephra::error::MatchBracketError;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::Success;
use tephra::Span;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Bracket combinators.
////////////////////////////////////////////////////////////////////////////////

pub fn bracket<'text: 'a, 'a, Sc, F, X: 'a, A>(
    open_tokens: &'a [Sc::Token],
    inner: F,
    close_tokens: &'a [Sc::Token],
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Option<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{ 
    map(bracket_index(open_tokens, inner, close_tokens, abort_pred),
        |(v, _)| v)
}


pub fn bracket_default<'text: 'a, 'a, Sc, F, X: 'a, A>(
    open_tokens: &'a [Sc::Token],
    inner: F,
    close_tokens: &'a [Sc::Token],
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, X> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{ 
    map(bracket_default_index(open_tokens, inner, close_tokens, abort_pred),
        |(v, _)| v)
}


/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens. Uses `Option` values for recoverable errors.
/// 
/// Each token in `open_tokens` is paired with the token of the same index in
/// `close_tokens`. Each slice must be the same length, and must not contain any
/// shared tokens.
///
/// The `abort_pred` argument can be used to limit the search for open
/// brackets by failing if any of the encountered tokens satisfies the predicate
/// before any open token is found.
///
/// ## Error recovery
///
/// Attempts error recovery if the given parser fails by scanning for the right
/// token. If the right token is not found, an unmatched delimiter error will
/// be emitted, and a `None` value will be returned.
pub fn bracket_index<'text: 'a, 'a, Sc, F, X: 'a, A>(
    open_tokens: &'a [Sc::Token],
    mut inner: F,
    close_tokens: &'a [Sc::Token],
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, (Option<X>, usize)> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{ 
    let option_inner = move |lexer, ctx| {
        (inner)
            (lexer, ctx)
            .map_value(Some)
    };

    bracket_default_index(open_tokens, option_inner, close_tokens, abort_pred)
}


/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens. Uses `Default` values for recoverable errors.
/// 
/// Each token in `open_tokens` is paired with the token of the same index in
/// `close_tokens`. Each slice must be the same length, and must not contain any
/// shared tokens.
///
/// The `abort_pred` argument can be used to limit the search space for brackets
/// on invalid inputs. If the `abort_pred` returns `true` for any token
/// encountered before the first open bracket, the search is aborted. (If a
/// `close_token` is encountered first, an error will be always be emitted.)
///
/// ## Error recovery
///
/// Attempts error recovery if the given parser fails by scanning for the right
/// token. If the right token is not found, an unmatched delimiter error will
/// be emitted, and a default value will be returned.
pub fn bracket_default_index<'text: 'a, 'a, Sc, F, X: 'a, A>(
    open_tokens: &'a [Sc::Token],
    mut inner: F,
    close_tokens: &'a [Sc::Token],
    abort_pred: A)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, (X, usize)> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{
    assert!(!(open_tokens.is_empty() || close_tokens.is_empty()),
        "invalid argument to bracket: empty token slice");
    assert!(open_tokens.len() == close_tokens.len() && !open_tokens.is_empty(), 
        "invalid argument to bracket: mismatched token slice lengths");
    for tok in open_tokens {
        assert!(!close_tokens.contains(tok), 
            "invalid argument to bracket: common open and close tokens \
            unsupported");
    }
    move |lexer, ctx| {
        let _trace_span = span!(Level::DEBUG, "bracket_*").entered();

        match match_nested_brackets(
            lexer.clone(),
            open_tokens,
            close_tokens,
            &abort_pred)
        {
            Err(bracket_error) => {
                event!(Level::DEBUG, "match nested brackets failed: {:?}",
                    bracket_error);
                Err(Box::new(bracket_error))
            },

            Ok(BracketMatch {mut open, mut close, index }) => {
                // Prepare the sublexers.
                let _ = open.next();
                let inner_lexer = open.into_sublexer();
                let _ = close.next();

                // NOTE: We will do manual error recovery here: we don't want to
                // advance to a token, because they're potentially nested. We
                // instead use the close token position we've already found, as
                // it is pointing to the proper recovery position.
                match (inner)
                    (inner_lexer, ctx.clone())
                    .map_value(|v| (v, index))
                {
                    Ok(succ) => {
                        event!(Level::DEBUG, "bracket inner parse success");
                        Ok(Success {
                            value: succ.value,
                            lexer: close,
                        })
                    },
                    Err(fail) => {
                        match ctx.send_error(fail) {
                            Err(parse_error) => {
                                event!(Level::DEBUG, "error recovery failed: \
                                    disabled");
                                Err(parse_error)
                            },

                            Ok(()) => {
                                event!(Level::DEBUG, "error recovery point \
                                    found");
                                Ok(Success {
                                    value: (X::default(), index),
                                    lexer: close,
                                })
                            },
                        }
                    },
                }
            },
        }
    }
}

/// Returns a `BracketMatch` object where the open lexer's `next` token is in
/// `open_tokens` and the close lexer's `next` token is the corresponding entry
/// in `right_tokens`.
///
/// Each token in `open_tokens` is paired with the token of the same index in
/// `close_tokens`. Each slice must be the same length, and must not contain any
/// shared tokens.
///
/// The `abort_pred` argument can be used to limit the search space for brackets
/// on invalid inputs. If the `abort_pred` returns `true` for any token
/// encountered before the first open bracket, the search is aborted. (If a
/// `close_token` is encountered first, an error will be always be emitted.)
#[allow(clippy::cognitive_complexity)]
fn match_nested_brackets<'text: 'a, 'a, Sc, A>(
    mut lexer: Lexer<'text, Sc>,
    open_tokens: &'a [Sc::Token],
    close_tokens: &'a [Sc::Token],
    abort_pred: A)
    -> Result<BracketMatch<'text, Sc>, MatchBracketError>
    where
        Sc: Scanner,
        A: Fn(&Sc::Token) -> bool + 'a + Clone,
{
    use MatchBracketError::*;
    let _trace_span = span!(Level::TRACE, "match_nested_*").entered();

    let start_span = Span::at(lexer.cursor_pos());
    let mut open_lexer: Option<Lexer<'_, Sc>> = None;
    // Detected open tokens as (index, count) pairs.
    let mut opened: SmallVec<[(usize, usize); DEFAULT_TOKEN_VEC_SIZE]>
        = SmallVec::with_capacity(open_tokens.len());

    while let Some(tok) = lexer.peek() {
        if let Some(idx) = close_tokens.iter().position(|t| t == &tok) {
            event!(Level::TRACE, "found close token ({:?} idx={} @ {})",
                tok, idx, lexer.cursor_pos());
            match opened.pop() {
                // Close token found before open token.
                None => return Err(Unopened {
                    found_end: lexer.peek_token_span().unwrap(),
                }),

                // Wrong close token for current open token.
                Some((t, _)) if t != idx => return Err(Mismatch {
                    found_start: open_lexer.unwrap().peek_token_span().unwrap(),
                    found_end: lexer.peek_token_span().unwrap(),
                }),

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
            event!(Level::TRACE, "found open token ({:?} idx={} @ {})",
                tok, idx, lexer.cursor_pos());
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
        } else if abort_pred(&tok) && open_lexer.is_none() {
            event!(Level::TRACE, "found abort token ({:?})", tok);
            // Abort search if no open tokens found before abort token.
            return Err(NoneFound {
                expected_start: lexer.peek_token_span().unwrap(),
            });
        }

        let _ = lexer.next();
    }

    match open_lexer {
        // End-of-text reached.
        None => Err(NoneFound {
            expected_start: start_span,
        }),
        // Unclosed open token.
        Some(open_lexer) => Err(Unclosed {
            found_start: open_lexer.peek_token_span().unwrap(),
        }),
    }
}


/// Default token buffer size for matching nested brackets.
const DEFAULT_TOKEN_VEC_SIZE: usize = 4;

/// Return value for `match_nested_brackets`.
struct BracketMatch<'text, Sc> where Sc: Scanner {
    /// The lexer whose `next` token is an open bracket.
    open: Lexer<'text, Sc>,
    /// The lexer whose `next` token is an close bracket.
    close: Lexer<'text, Sc>,
    /// The index of the brackets that were matched.
    index: usize,
}

