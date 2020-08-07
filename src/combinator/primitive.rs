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
use crate::span::NewLine;
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
pub fn empty<'text, Sc, Nl>(lexer: Lexer<'text, Sc, Nl>)
    -> ParseResult<'text, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    Ok(Success {
        lexer,
        value: (),
    })
}


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
pub fn end_of_text<'text, Sc, Nl, F, V>(lexer: Lexer<'text, Sc, Nl>)
    -> ParseResult<'text, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    if lexer.is_empty() {
        Ok(Success {
            lexer,
            value: (),
        })
    } else {
        Err(Failure {
            parse_error: ParseError::unexpected_text(lexer.end_span()),
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
pub fn one<'text, Sc, Nl>(token: Sc::Token)
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Sc::Token>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    move |mut lexer| {
        // print!("one({:?})", token);
        if lexer.is_empty() {
            // Unexpected End-of-text.
            return Err(Failure {
                parse_error: ParseError::unexpected_end_of_text(lexer.end_span()),
                lexer,
                source: None,
            });
        }

        match lexer.next() {
            // Lexer error.
            None => {
                // println!(" -> unrecognized {}", lexer.last_span());
                Err(Failure {
                    parse_error: ParseError::unrecognized_token(lexer.end_span()),
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
                    parse_error: ParseError::unexpected_token(lexer.last_span()),
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
pub fn any<'text, Sc, Nl>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, Sc::Token>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {
        for token in &tokens {
            match lexer.next() {
                // Lexer error.
                None => return Err(Failure {
                    parse_error: ParseError::unrecognized_token(lexer.last_span()),
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
            parse_error: ParseError::unexpected_token(lexer.last_span()),
            lexer,
            source: None,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// seq
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the success only if each succeeds.
pub fn seq<'text, Sc, Nl>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'text, Sc, Nl>) -> ParseResult<'text, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {
        for token in &tokens {
            if lexer.is_empty() {
                // Unexpected End-of-text.
                return Err(Failure {
                    parse_error: ParseError::unexpected_end_of_text(lexer.end_span()),
                    lexer,
                    source: None,
                });
            }

            match lexer.next() {
                // Lexer error.
                None => return Err(Failure {
                    parse_error: ParseError::unrecognized_token(lexer.last_span()),
                    lexer,
                    source: None,
                }),

                // Matching token.
                Some(lex) if lex == *token => (),

                // Incorrect token.
                Some(_) => return Err(Failure {
                    parse_error: ParseError::unexpected_token(lexer.last_span()),
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
