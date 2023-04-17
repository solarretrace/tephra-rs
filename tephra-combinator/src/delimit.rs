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
use tephra::Context;
use tephra::Lexer;
use tephra::Failure;
use tephra::Highlight;
use tephra::Success;
use tephra::Span;
use tephra::SpanDisplay;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Scanner;
use tephra::ParseError;
use tephra_tracing::Level;
use tephra_tracing::span;
use smallvec::SmallVec;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Miscellaneous delimiter combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which requires the given parser to produce a value by
/// consuming up to one of the given tokens.
///
/// If the given parser is successful, but doesn't terminate on one of the
/// `abort_tokens`, then an error is returned.
///
/// ## Error recovery
///
/// No error recovery is attempted.
pub fn up_to<'text: 'a, 'a, Sc, F, X: 'a>(
    mut parser: F,
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, X> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
{
    move |lexer, ctx| { 
        let mut succ = (parser)(lexer, ctx)?;

        match succ.lexer.peek() {
            None                                     => Ok(succ),
            Some(tok) if abort_tokens.contains(&tok) => Ok(succ),
            _ => {
                // Advance lexer to the expected token.
                succ.lexer.advance_to(abort_tokens);

                Err(Failure {
                    parse_error: ParseError::new(
                        succ.lexer.source_text(),
                        "incomplete parse"),
                    lexer: succ.lexer,
                })
            },
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Bracket combinators.
////////////////////////////////////////////////////////////////////////////////


/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens. Uses `Option` values for recoverable errors.
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
/// be emitted, and a `None` value will be returned.
pub fn bracket<'text: 'a, 'a, Sc, F, X: 'a>(
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
    let option_center = move |lexer, ctx| {
        (center)
            (lexer, ctx)
            .map_value(Some)
    };

    bracket_default(open_tokens, option_center, close_tokens, abort_tokens)
}

/// Returns a parser which brackets the given parser in a any of a given pair of
/// matching tokens. Uses `Default` values for recoverable errors.
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
/// be emitted, and a default value will be returned.
pub fn bracket_default<'text: 'a, 'a, Sc, F, X: 'a>(
    open_tokens: &'a [Sc::Token],
    mut center: F,
    close_tokens: &'a [Sc::Token],
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, (X, usize)> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
{
    if open_tokens.is_empty() || close_tokens.is_empty() {
        panic!("invalid argument to bracket: empty token slice");
    }
    if open_tokens.len() != close_tokens.len() || open_tokens.is_empty() {
        panic!("invalid argument to bracket: mismatched token slice \
            lengths");
    }
    for tok in open_tokens {
        if close_tokens.contains(tok) {
            panic!("invalid argument to bracket: common open and close \
                tokens unsupported");       
        }
    }

    use BracketError::*;

    move |lexer, ctx| {
        let _span = span!(Level::DEBUG, "bracket").entered();

        match match_nested_brackets(lexer.clone(),
            open_tokens,
            close_tokens,
            abort_tokens)
        {
            Err(NoneFound(lexer)) => {
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "expected open bracket")
                        .with_span_display(SpanDisplay::new_error_highlight(
                            lexer.source_text(),
                            lexer.start_span(),
                            "bracket expected here")),
                    lexer,
                })
            },
            Err(Unopened(lexer)) => {
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "unmatched close bracket")
                        .with_span_display(SpanDisplay::new_error_highlight(
                            lexer.source_text(),
                            lexer.peek_token_span().unwrap(),
                            "this bracket has no matching open")),
                    lexer,
                })
            },
            Err(Unclosed(lexer)) => {
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "unmatched open bracket")
                        .with_span_display(SpanDisplay::new_error_highlight(
                            lexer.source_text(),
                            lexer.peek_token_span().unwrap(),
                            "this bracket is not closed")),
                    lexer,
                })
            },
            Err(Mismatch(open_lexer, close_lexer)) => {
                Err(Failure {
                    parse_error: ParseError::new(
                            lexer.source_text(),
                            "unmatched brackets")
                        .with_span_display(SpanDisplay::new_error_highlight(
                            lexer.source_text(),
                            open_lexer.peek_token_span().unwrap(),
                            "the bracket here")
                            .with_highlight(Highlight::new(
                                close_lexer.peek_token_span().unwrap(),
                                "... does not match the closing bracket here")
                                .with_error_type())),
                    lexer,
                })
            },

            Ok(BracketMatch {mut open, mut close, index }) => {
                // Prepare the sublexers.
                let _ = open.next();
                let center_lexer = open.sublexer();
                let _ = close.next();

                // NOTE: We will do manual error recovery here: we don't want to
                // advance to a token, because they're potentially nested. We
                // instead substitute the close, which is already pointing
                // to the proper recovery position.
                match (center)
                    (center_lexer, ctx.clone())
                    .map_value(|v| (v, index))
                {
                    Ok(succ) => Ok(Success {
                        value: succ.value,
                        lexer: close,
                    }),
                    Err(fail) => {
                        match ctx.send_error(fail.parse_error, &fail.lexer) {
                            Err(parse_error) => Err(Failure {
                                parse_error,
                                lexer: close,
                            }),
                            Ok(()) => Ok(Success {
                                value: (X::default(), index),
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

////////////////////////////////////////////////////////////////////////////////
// List combinators.
////////////////////////////////////////////////////////////////////////////////

pub fn delimited_list<'text: 'a, 'a, Sc, F, X: 'a>(
    parser: F,
    sep_token: Sc::Token,
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<Option<X>>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
{
    delimited_list_bounded(0, None, parser, sep_token, abort_tokens)
}


pub fn delimited_list_bounded<'text: 'a, 'a, Sc, F, X: 'a>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    sep_token: Sc::Token,
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<Option<X>>> + 'a
    where
        Sc: Scanner + 'a,
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
        abort_tokens)
}


pub fn delimited_list_default<'text: 'a, 'a, Sc, F, X: 'a>(
    parser: F,
    sep_token: Sc::Token,
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
{
    delimited_list_bounded_default(0, None, parser, sep_token, abort_tokens)
}


pub fn delimited_list_bounded_default<'text: 'a, 'a, Sc, F, X: 'a>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    sep_token: Sc::Token,
    abort_tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<X>> + 'a
    where
        Sc: Scanner + 'a,
        F: FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
            -> ParseResult<'text, Sc, X> + 'a,
        X: Default,
{
    let mut sep_or_abort_tokens = Vec::with_capacity(abort_tokens.len() + 1);
    sep_or_abort_tokens.push(sep_token.clone());
    sep_or_abort_tokens.extend_from_slice(abort_tokens);

    let recover_abort_tokens = sep_or_abort_tokens.to_owned();
    let recover_pat = Rc::new(RwLock::new(move |tok| {
        Ok(recover_abort_tokens.contains(&tok))
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

        let start_position = lexer.cursor_pos();
        let mut aborting = false;
        
        loop {
            // End loop if no remaining text.
            match lexer.peek() {
                None => { break; }
                Some(tok) if abort_tokens.contains(&tok) => {
                    if vals.is_empty() { break; }
                    aborting = true;
                }
                _ => (),
            }
            if lexer.is_empty() { break; }

            // Try to parse value.
            let res = stabilize(recover_default(
                    up_to(&mut parser, &sep_or_abort_tokens),
                    recover_pat.clone()))
                (lexer.clone(), ctx.clone());

            if aborting {
                vals.push(X::default());
                break;
            }
            let (val, succ) = res?
                .take_value();
            lexer = succ.lexer.sublexer();
            vals.push(val);

            // End loop if high is reached.
            if let Some(n) = high { if vals.len() >= n { break; } }
            // End loop if no remaining text.
            match lexer.peek() {
                None => { break; }
                Some(tok) if abort_tokens.contains(&tok) => { break; }
                _ => (),
            }
            if lexer.is_empty() { break; }

            // Try to parse sep token.
            let (_, succ) = stabilize(recover_default(
                    discard(one(sep_token.clone())),
                    recover_pat.clone()))
                (lexer.clone(), ctx.clone())?
                .take_value();
            lexer = succ.lexer.sublexer();
        }

        if vals.len() < low {
            let parse_error = ParseError::new(
                    lexer.source_text(),
                    "expected more items in list")
                .with_span_display(SpanDisplay::new_error_highlight(
                    lexer.source_text(),
                    Span::new_enclosing(start_position, lexer.cursor_pos()),
                    format!("expected {low} item{}, found {}",
                        if vals.len() > 0 { "s" } else { "" },
                        vals.len())));
                
            match ctx.send_error(parse_error, &lexer) {
                Err(parse_error) => Err(Failure {
                    parse_error,
                    lexer,
                }),
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
