////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Combinator primitives.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use tephra::lexer::Lexer;
use tephra::lexer::Scanner;
use tephra::result::Failure;
use tephra::result::ParseError;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tephra::result::Success;

// External library imports.
use tracing::event;
use tracing::Level;
use tracing::span;



////////////////////////////////////////////////////////////////////////////////
// empty
////////////////////////////////////////////////////////////////////////////////
/// Parses the empty string.
///
/// # Combinator behavior
///
/// ## Success
/// 
/// The lexer is returned with no tokens requested or span consumed.
///
/// ## Failure
///
/// This parser cannot fail.
pub fn empty<'text, Sc>(lexer: Lexer<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    Ok(Success {
        lexer,
        value: (),
    })
}

/// Parses any token and fails. Useful for failing if a peeked token doesn't
/// match any expected tokens.
#[tracing::instrument(level = "debug")]
pub fn fail<'text, Sc>(mut lexer: Lexer<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    match lexer.next() {
        Some(token) => {
            event!(Level::TRACE, "success converted to failure");
            Err(Failure {
                parse_error: ParseError::unexpected_token(
                    lexer.token_span(),
                    &token,
                    lexer.column_metrics()),
                lexer,
                source: None,
            })
        },
        None => {
            event!(Level::TRACE, "no tokens");
            Err(Failure {
                parse_error: ParseError::unexpected_end_of_text(
                    lexer.end_span(),
                    lexer.column_metrics()),
                lexer,
                source: None,
            })
        },
    }
}

/// Returns a parser which converts any failure into an empty success.
pub fn maybe<'text, Sc, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Option<V>>
    where
        Sc: Scanner,
        F: FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, V>,
{
    move |lexer| {
        let _span = span!(Level::DEBUG, "maybe").entered();

        let initial = lexer.clone();
        
        match parser
            (lexer)
            .trace_result(Level::TRACE, "subparse")
        {
            Ok(succ) => Ok(succ.map_value(Some)),

            Err(_)   => Ok(Success {
                lexer: initial,
                value: None,
            }),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
#[tracing::instrument(level = "debug")]
pub fn end_of_text<'text, Sc>(lexer: Lexer<'text, Sc>)
    -> ParseResult<'text, Sc, ()>
    where Sc: Scanner,
{
    if lexer.is_empty() {
        event!(Level::TRACE, "end of text found");
        Ok(Success {
            lexer,
            value: (),
        })
    } else {
        event!(Level::TRACE, "end of text not found");
        Err(Failure {
            parse_error: ParseError::expected_end_of_text(
                lexer.end_span(),
                lexer.column_metrics()),
            lexer,
            source: None,
        })
    }

}

////////////////////////////////////////////////////////////////////////////////
// one
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which consumes a single token if it matches the given
/// token.
pub fn one<'text, Sc>(token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Sc::Token>
    where Sc: Scanner,
{

    move |mut lexer| {
        let _span = span!(Level::DEBUG, "one", expected = ?token).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        if lexer.is_empty() {
            event!(Level::TRACE, "lexer is empty");
            // Unexpected End-of-text.
            return Err(Failure {
                parse_error: ParseError::unexpected_end_of_text(
                    lexer.end_span(),
                    lexer.column_metrics()),
                lexer,
                source: None,
            });
        }

        match lexer.next() {
            // Lexer error.
            None => {
                event!(Level::TRACE, "lexer error");
                Err(Failure {
                    parse_error: ParseError::unrecognized_token(
                        lexer.token_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                })
            },

            // Matching token.
            Some(lex) if lex == token => {
                event!(Level::TRACE, "correct token {{found={:?}}}", lex);
                Ok(Success {
                    lexer,
                    value: token.clone(),
                })
            },

            // Incorrect token.
            Some(lex) => {
                event!(Level::TRACE, "incorrect token {{found={:?}}}", lex);
                Err(Failure {
                    parse_error: ParseError::unexpected_token(
                        lexer.token_span(),
                        &token,
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                })
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// any
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the first which succeeds.
pub fn any<'text, Sc>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Sc::Token>
    where Sc: Scanner,
{
    let tokens = tokens.to_vec();
    move |lexer| {
        let _span = span!(Level::DEBUG, "any",
            expected = ?DisplayList(&tokens[..])).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        for token in &tokens {
            let _span = span!(Level::TRACE, "any", expect = ?token).entered();

            let mut lexer = lexer.clone();
            match lexer.next() {
                // Lexer error.
                None => return Err(Failure {
                    parse_error: ParseError::unrecognized_token(
                        lexer.token_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                }),

                // Matching token.
                Some(lex) if lex == *token => return Ok(Success {
                    lexer,
                    value: token.clone(),
                }),

                // Incorrect token.
                // Unexpected End-of-text.
                _ => (),
            }
        }

        Err(Failure {
            parse_error: ParseError::unexpected_token(
                lexer.token_span(),
                format!("one of {}", DisplayList(&tokens[..])),
                lexer.column_metrics()),
            lexer,
            source: None,
        })
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
pub fn seq<'text, Sc>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc>) -> ParseResult<'text, Sc, Vec<Sc::Token>>
    where Sc: Scanner,
{
    let tokens = tokens.to_vec();
    let cap = tokens.len();
    move |mut lexer| {
        let _span = span!(Level::DEBUG, "seq",
            expected = ?DisplayList(&tokens[..])).entered();

        event!(Level::TRACE, "before parse:\n{}", lexer);

        let mut found = Vec::with_capacity(cap);
        
        for token in &tokens {
            let _span = span!(Level::TRACE, "seq", expect = ?token).entered();

            if lexer.is_empty() {
                // Unexpected End-of-text.
                return Err(Failure {
                    parse_error: ParseError::unexpected_end_of_text(
                        lexer.end_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                });
            }

            match lexer.next() {
                // Lexer error.
                None => return Err(Failure {
                    parse_error: ParseError::unrecognized_token(
                        lexer.token_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                }),

                // Matching token.
                Some(lex) if lex == *token => found.push(lex),

                // Incorrect token.
                Some(_) => return Err(Failure {
                    parse_error: ParseError::unexpected_token(
                        lexer.token_span(),
                        &token,
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                }),
            }
        }

        Ok(Success {
            lexer,
            value: found,
        })
    }
}
