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
use tephra::Lexer;
use tephra::ParseResult;
use tephra::Scanner;
use tephra::Success;

// External library imports.
use simple_predicates::DnfVec;
use simple_predicates::Eval;
use simple_predicates::Expr;
use tephra::error::Expected;
use tephra::error::Found;
use tephra::error::UnexpectedTokenError;
use tephra::error::UnrecognizedTokenError;
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
    event!(Level::TRACE, "empty: Success");
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
        let _trace_span = span!(Level::TRACE, "one").entered();
        let error_span = lexer.parse_span();

        match lexer.next() {
            // Matching token.
            Some(lex) if lex == token => {
                event!(Level::TRACE, "success ({:?})", lex);
                Ok(Success {
                    lexer,
                    value: lex,
                })
            },

            // Incorrect token.
            #[cfg_attr(not(feature="tracing"), allow(unused_variables))]
            Some(lex) => {
                event!(Level::TRACE, "UnexpectedTokenError ({:?})", lex);
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::Token(token.clone()),
                    found: Found::Token(lex),
                }))
            },

            // Unexpected end-of-text.
            None => {
                event!(Level::DEBUG, "UnexpectedTokenError (EndOfText)");
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::Token(token.clone()),
                    found: Found::EndOfText,
                }))
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
    assert!(!tokens.is_empty(), "empty token slice not supported");

    move |mut lexer, _ctx| {
        let _trace_span = span!(Level::TRACE, "any").entered();
        let error_span = lexer.parse_span();

        match lexer.peek() {
            Some(lex) => {
                for token in tokens {
                    if lex == *token {
                        let _ = lexer.next();
                        event!(Level::TRACE, "success ({:?})", lex);
                        return Ok(Success {
                            value: token.clone(),
                            lexer,
                        });
                    }
                }

                event!(Level::TRACE, "UnexpectedTokenError ({:?})", lex);
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::any(tokens.iter().cloned()),
                    found: Found::Token(lex),
                }))
            }

            // Unexpected end-of-text.
            None => {
                event!(Level::DEBUG, "UnexpectedTokenError (EndOfText)");
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::any(tokens.iter().cloned()),
                    found: Found::EndOfText,
                }))
            },
        }
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
    assert!(!tokens.is_empty(), "empty token slice not supported");

    move |mut lexer, _ctx| {
        let _trace_span = span!(Level::TRACE, "any_index").entered();
        let error_span = lexer.parse_span();

        match lexer.peek() {
            Some(lex) => {
                for (idx, token) in tokens.iter().enumerate() {
                    if lex == *token {
                        let _ = lexer.next();
                        event!(Level::TRACE, "success ({:?}@{:?})", lex, idx);
                        return Ok(Success {
                            value: idx,
                            lexer,
                        });
                    }
                }

                event!(Level::TRACE, "UnexpectedTokenError ({:?})", lex);
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::any(tokens.iter().cloned()),
                    found: Found::Token(lex),
                }))
            }

            // Unexpected end-of-text.
            None => {
                event!(Level::DEBUG, "UnexpectedTokenError (EndOfText)");
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::any(tokens.iter().cloned()),
                    found: Found::EndOfText,
                }))
            },
        }
    }
}

/// Struct for displaying tokens in `any` errors.
struct DisplayList<'a, T>(&'a[T]);

impl<'a, T> std::fmt::Display for DisplayList<'a, T>
    where T: std::fmt::Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, val) in self.0.iter().enumerate() {
            write!(f, "{val}")?;
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
            write!(f, "{val:?}")?;
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
    // is potential value in constructing the result incrementally as we do.

    let cap = tokens.len();
    move |mut lexer, _ctx| {
        let _trace_span = span!(Level::TRACE, "seq").entered();
        let error_span = lexer.parse_span();
        let mut found = Vec::with_capacity(cap);
        
        for token in tokens {
            match lexer.next() {
                // Matching token.
                Some(lex) if lex == *token => {
                    event!(Level::TRACE, "found ({:?})", lex);
                    found.push(lex)
                },

                // Incorrect token.
                Some(lex) => {
                    event!(Level::TRACE, "UnexpectedTokenError ({:?})", lex);
                    return Err(Box::new(UnexpectedTokenError {
                        error_span,
                        token_span: lexer.token_span(),
                        expected: Expected::Token(token.clone()),
                        found: Found::Token(lex),
                    }));
                }

                // Unexpected end-of-text.
                None => {
                    event!(Level::DEBUG, "UnexpectedTokenError (EndOfText)");
                    return Err(Box::new(UnexpectedTokenError {
                        error_span,
                        token_span: lexer.token_span(),
                        expected: Expected::Token(token.clone()),
                        found: Found::EndOfText,
                    }));
                },
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
        let _trace_span = span!(Level::TRACE, "seq_count").entered();
        let error_span = lexer.parse_span();
        
        let mut count = 0;
        for token in tokens {
            if lexer.is_empty() { break }

            match lexer.peek() {
                // Matching token.
                Some(lex) if lex == *token => {
                    event!(Level::TRACE, "found ({:?})", lex);
                    count += 1;
                    let _ = lexer.next();
                }

                // Incorrect token.
                Some(_) => break,

                // Unrecognized token.
                None => {
                    event!(Level::ERROR, "UnrecognizedTokenError: {}", lexer);
                    return Err(Box::new(UnrecognizedTokenError {
                        error_span,
                    }))
                },
            }
        }

        event!(Level::TRACE, "success (matched {:?} tokens)", count);
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
        let error_span = lexer.parse_span();

        match lexer.next() {
            // Unexpected end-of-text.
            None => {
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::<Sc::Token>::Other(
                        format!("{pred:?}")),
                    found: Found::EndOfText,
                }))
            },

            // Matching token.
            Some(lex) if pred.eval(&lex) => {
                Ok(Success {
                    lexer,
                    value: lex,
                })
            },

            // Incorrect token.
            // TODO: Better predicate formatting.
            Some(lex) => {
                Err(Box::new(UnexpectedTokenError {
                    error_span,
                    token_span: lexer.token_span(),
                    expected: Expected::Other(format!("{pred:?}")),
                    found: Found::Token(lex),
                }))
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
    mut lexer: Lexer<'text, Sc>,
    _ctx: Context<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    let error_span = lexer.parse_span();

    if lexer.is_empty() {
        event!(Level::TRACE, "end_of_text: Success");
        Ok(Success {
            lexer,
            value: (),
        })
    } else {
        let lex = lexer.peek().unwrap();

        event!(Level::ERROR, "end_of_text: UnexpectedTokenError {}", lexer);
        Err(Box::new(UnexpectedTokenError {
            error_span,
            token_span: lexer.token_span(),
            expected: Expected::EndOfText,
            found: Found::Token(lex),
        }))
    }
}
