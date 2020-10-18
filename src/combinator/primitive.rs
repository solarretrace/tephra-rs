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
use crate::position::ColumnMetrics;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseResult;
use crate::result::Success;
use crate::result::Failure;
use crate::result::ParseError;


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
pub fn empty<'text, Sc, Cm>(lexer: Lexer<'text, Sc, Cm>)
    -> ParseResult<'text, Sc, Cm, ()>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    Ok(Success {
        lexer,
        value: (),
    })
}

/// Parses any token and fails. Useful for failing if a peeked token doesn't
/// match any expected tokens.
pub fn fail<'text, Sc, Cm>(mut lexer: Lexer<'text, Sc, Cm>)
    -> ParseResult<'text, Sc, Cm, ()>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    match lexer.next() {
        Some(token) => Err(Failure {
            parse_error: ParseError::unexpected_token(
                lexer.last_span(),
                &token,
                lexer.column_metrics()),
            lexer,
            source: None,
        }),
        None => Err(Failure {
            parse_error: ParseError::unexpected_end_of_text(
                lexer.end_span(),
                lexer.column_metrics()),
            lexer,
            source: None,
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
pub fn end_of_text<'text, Sc, Cm>(mut lexer: Lexer<'text, Sc, Cm>)
    -> ParseResult<'text, Sc, Cm, ()>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    lexer.filter_next();
    if lexer.is_empty() {
        Ok(Success {
            lexer,
            value: (),
        })
    } else {
        Err(Failure {
            parse_error: ParseError::unexpected_text(
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
pub fn one<'text, Sc, Cm>(token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Sc::Token>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    move |mut lexer| {
        // print!("one({:?})", token);
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
            None => {
                // println!(" -> unrecognized {}", lexer.last_span());
                Err(Failure {
                    parse_error: ParseError::unrecognized_token(
                        lexer.last_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                })
            },

            // Matching token.
            Some(lex) if lex == token => {
                // println!(" -> MATCH {}", lexer.last_span());
                Ok(Success {
                    lexer,
                    value: token.clone(),
                })
            },

            // Incorrect token.
            Some(_) => {
                // println!( " -> unexpected {}", lexer.last_span());
                Err(Failure {
                    parse_error: ParseError::unexpected_token(
                        lexer.last_span(),
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
pub fn any<'text, Sc, Cm>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, Sc::Token>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {
        for token in &tokens {
            match lexer.next() {
                // Lexer error.
                None => return Err(Failure {
                    parse_error: ParseError::unrecognized_token(
                        lexer.last_span(),
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
            lexer.reset();
        }

        Err(Failure {
            parse_error: ParseError::unexpected_token(
                lexer.last_span(),
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


////////////////////////////////////////////////////////////////////////////////
// seq
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the success only if each succeeds.
pub fn seq<'text, Sc, Cm>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc, Cm>) -> ParseResult<'text, Sc, Cm, ()>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {
        for token in &tokens {
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
                        lexer.last_span(),
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                }),

                // Matching token.
                Some(lex) if lex == *token => (),

                // Incorrect token.
                Some(_) => return Err(Failure {
                    parse_error: ParseError::unexpected_token(
                        lexer.last_span(),
                        &token,
                        lexer.column_metrics()),
                    lexer,
                    source: None,
                }),
            }
        }

        Ok(Success {
            lexer,
            value: (),
        })
    }
}
