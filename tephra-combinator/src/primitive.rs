////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Combinator primitives.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use tephra::Context;
use tephra::Failure;
use tephra::Lexer;
use tephra::ParseError;
use tephra::ParseResult;
use tephra::Scanner;
use tephra::Success;

// External library imports.
use simple_predicates::DnfVec;
use simple_predicates::Eval;
use simple_predicates::Expr;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;



////////////////////////////////////////////////////////////////////////////////
// empty
////////////////////////////////////////////////////////////////////////////////
/// Parses the empty string.
///
/// ## Combinator behavior
///
/// ### Success
/// 
/// The lexer is returned with no tokens requested or span consumed.
///
/// ### Failure
///
/// This parser cannot fail.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn empty<'text, Sc>(lexer: Lexer<'text, Sc>, _ctx: Context<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    Ok(Success {
        lexer,
        value: (),
    })
}

// TODO: Make a version of this to consume filtered tokens? any_filtered


////////////////////////////////////////////////////////////////////////////////
// one
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which consumes a single token if it matches the given
/// token.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn one<'text, Sc>(token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Sc::Token>
    where Sc: Scanner,
{

    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "one", expected = ?token).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        match lexer.next() {
            // Unexpected end-of-text.
            None => {
                event!(Level::TRACE, "lexer is empty");
                Err(Failure::new(
                    ParseError::unexpected_end_of_text(
                        lexer.source_text(),
                        lexer.end_span()),
                    lexer
                ))
            },

            // Matching token.
            Some(lex) if lex == token => {
                event!(Level::TRACE, "correct token {{found={:?}}}", lex);
                Ok(Success {
                    lexer,
                    value: lex,
                })
            },

            // Incorrect token.
            #[cfg_attr(not(feature="tracing"), allow(unused_variables))]
            Some(lex) => {
                event!(Level::TRACE, "incorrect token {{found={:?}}}", lex);
                Err(Failure::new(
                    ParseError::unexpected_token(
                        lexer.source_text(),
                        lexer.token_span(),
                        &token),
                    lexer,
                ))
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// any
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which attempts to match each of the given tokens in
/// sequence, returning the first which succeeds.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn any<'text, 'a, Sc>(tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Sc::Token> + 'a
    where Sc: Scanner,
{
    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "any",
            expected = ?DisplayList(tokens)).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        for token in tokens {
            let _span = span!(Level::TRACE, "any", expect = ?token).entered();

            match lexer.peek() {
                // Unexpected end-of-text.
                None => {
                    event!(Level::TRACE, "lexer is empty");
                    return Err(Failure::new(
                        ParseError::unexpected_end_of_text(
                            lexer.source_text(),
                            lexer.end_span()),
                        lexer
                    ));
                },

                // Matching token.
                Some(lex) if lex == *token => {
                    lexer.next();
                    return Ok(Success {
                        value: token.clone(),
                        lexer,
                    })
                },

                // Incorrect token.
                // Unexpected End-of-text.
                _ => (),
            }
        }

        Err(Failure::new(
            ParseError::unexpected_token(
                lexer.source_text(),
                lexer.token_span(),
                format!("one of {}", DisplayList(&tokens[..]))),
            lexer
        ))
    }
}

/// Returns a parser which attempts to match each of the given tokens in
/// sequence, returning the index of the first which succeeds.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn any_index<'text, 'a, Sc>(tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize> + 'a
    where Sc: Scanner,
{
    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "any",
            expected = ?DisplayList(&tokens[..])).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        for (idx, token) in tokens.iter().enumerate() {
            let _span = span!(Level::TRACE, "any", expect = ?token).entered();

            match lexer.peek() {
                // Unexpected end-of-text.
                None => {
                    event!(Level::TRACE, "lexer is empty");
                    return Err(Failure::new(
                        ParseError::unexpected_end_of_text(
                            lexer.source_text(),
                            lexer.end_span()),
                        lexer
                    ));
                },

                // Matching token.
                Some(lex) if lex == *token => {
                    lexer.next();
                    return Ok(Success {
                        value: idx,
                        lexer,
                    })
                },

                // Incorrect token.
                // Unexpected End-of-text.
                _ => (),
            }
        }

        Err(Failure::new(
            ParseError::unexpected_token(
                lexer.source_text(),
                lexer.token_span(),
                format!("one of {}", DisplayList(&tokens[..]))),
            lexer
        ))
    }
}

/// Struct for displaying tokens in `any` errors.
struct DisplayList<'a, T>(&'a[T]);

impl<'a, T> std::fmt::Display for DisplayList<'a, T>
    where T: std::fmt::Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, val) in self.0.iter().enumerate() {
            write!(f, "{}", val)?;
            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

impl<'a, T> std::fmt::Debug for DisplayList<'a, T>
    where T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, val) in self.0.iter().enumerate() {
            write!(f, "{:?}", val)?;
            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}



////////////////////////////////////////////////////////////////////////////////
// seq
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the success only if each succeeds.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn seq<'text, 'a, Sc>(tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Vec<Sc::Token>> + 'a
    where Sc: Scanner,
{
    // NOTE: We could just preconstruct the result here, since if `seq` succeeds
    // then the result is the input. However, it is possible that `PartialEq` on
    // the tokens allows two tokens to match when they are different, so there
    // is potential value in constructing the result incrementally like this.

    let cap = tokens.len();
    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "seq",
            expected = ?DisplayList(tokens)).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        let mut found = Vec::with_capacity(cap);
        
        for token in tokens {
            let _span = span!(Level::TRACE, "seq", expect = ?token).entered();

            match lexer.next() {
                // Unexpected end-of-text.
                None => return Err(Failure::new(
                    ParseError::unexpected_end_of_text(
                        lexer.source_text(),
                        lexer.end_span()),
                    lexer
                )),

                // Matching token.
                Some(lex) if lex == *token => found.push(lex),

                // Incorrect token.
                Some(_) => return Err(Failure::new(
                    ParseError::unexpected_token(
                        lexer.source_text(),
                        lexer.token_span(),
                        &token),
                    lexer
                )),
            }
        }

        Ok(Success {
            lexer,
            value: found,
        })
    }
}

/// Returns a parser attempts each of the given tokens in sequence, returning
/// the number of tokens successfully parsed.
/// 
/// This combinator will only fail on an unrecognized token.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn seq_count<'text, 'a, Sc>(tokens: &'a [Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, usize> + 'a
    where Sc: Scanner,
{
    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "seq_count",
            expected = ?DisplayList(tokens)).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);
        
        let mut count = 0;
        for token in tokens {
            let _span = span!(Level::TRACE, "seq_count", expect = ?token)
                .entered();

            if lexer.is_empty() { break }

            match lexer.peek() {
                // Unexpected end-of-text.
                None => return Err(Failure::new(
                    ParseError::unrecognized_token(
                        lexer.source_text(),
                        lexer.end_span()),
                    lexer
                )),

                // Matching token.
                Some(lex) if lex == *token => {
                    count += 1;
                    lexer.next();
                }

                // Incorrect token.
                Some(_) => break,
            }
        }

        Ok(Success {
            lexer,
            value: count,
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// pred
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which consumes a single token if it satisfies the given
/// token predicate.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn pred<'text, Sc>(expr: Expr<Sc::Token>)
    -> impl FnMut(Lexer<'text, Sc>, Context<'text, Sc>)
        -> ParseResult<'text, Sc, Sc::Token>
    where Sc: Scanner,
{
    let pred = DnfVec::from(expr.map(Token));
    move |mut lexer, _ctx| {
        let _span = span!(Level::DEBUG, "one", expected = ?token).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        match lexer.next() {
            // Unexpected end-of-text.
            None => {
                event!(Level::TRACE, "lexer error");
                Err(Failure::new(
                    ParseError::unexpected_end_of_text(
                        lexer.source_text(),
                        lexer.end_span()),
                    lexer
                ))
            },

            // Matching token.
            Some(lex) if pred.eval(&lex) => {
                event!(Level::TRACE, "correct token {{found={:?}}}", lex);
                Ok(Success {
                    lexer,
                    value: lex,
                })
            },

            // Incorrect token.
            // TODO: Better predicate formatting.
            #[cfg_attr(not(feature="tracing"), allow(unused_variables))]
            Some(lex) => {
                event!(Level::TRACE, "incorrect token {{found={:?}}}", lex);
                Err(Failure::new(
                    ParseError::unexpected_token(
                        lexer.source_text(),
                        lexer.token_span(),
                        format!("{:?}", pred)),
                    lexer,
                ))
            },
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Token<T>(T);

impl<T> Eval for Token<T> where T: Clone + PartialEq {
    type Context = T;

    fn eval(&self, data: &Self::Context) -> bool {
        &self.0 == data
    }
}


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn end_of_text<'text, Sc>(
    lexer: Lexer<'text, Sc>,
    _ctx: Context<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    let _span = span!(Level::DEBUG, "end_of_text").entered();

    if lexer.is_empty() {
        event!(Level::TRACE, "end of text found");
        Ok(Success {
            lexer,
            value: (),
        })
    } else {
        event!(Level::TRACE, "end of text not found");
        Err(Failure::new(
            ParseError::expected_end_of_text(
                lexer.source_text(),
                lexer.end_span()),
            lexer
        ))
    }
}


////////////////////////////////////////////////////////////////////////////////
// fail
////////////////////////////////////////////////////////////////////////////////
// TODO: Make this more general by taking a ParseError instead?
/// Parses any token and fails. Useful for failing if a peeked token doesn't
/// match any expected tokens.
///
/// ### Error recovery
///
/// No error recovery is attempted.
pub fn fail<'text, Sc, V>(
    mut lexer: Lexer<'text, Sc>,
    _ctx: Context<'text, Sc>)
    -> ParseResult<'text, Sc, V>
    where Sc: Scanner,
{
    let _span = span!(Level::DEBUG, "fail").entered();

    match lexer.next() {
        Some(token) => {
            event!(Level::TRACE, "success converted to failure");
            Err(Failure::new(
                ParseError::unexpected_token(
                    lexer.source_text(),
                    lexer.token_span(),
                    &token),
                lexer
            ))
        },
        None => {
            event!(Level::TRACE, "no tokens");
            Err(Failure::new(
                ParseError::unexpected_end_of_text(
                    lexer.source_text(),
                    lexer.end_span()),
                lexer
            ))
        },
    }
}

