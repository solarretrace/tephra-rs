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
use crate::result::Reason;

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
pub fn empty<'t, Sc, Nl>(lexer: Lexer<'t, Sc, Nl>)
    -> ParseResult<'t, Sc, Nl, ()>
    where Sc: Scanner,
{

    Ok(Success {
        lexer: lexer,
        value: (),
    })
}


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
pub fn end_of_text<'t, Sc, Nl, F, V>(mut lexer: Lexer<'t, Sc, Nl>)
    -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    match lexer.next() {
        // Lexer error.
        Some(Err(e)) => Err(Failure {
            lexer,
            reason: Reason::LexerError,
            source: Some(Box::new(e)),
        }),

        // Expected End-of-text.
        None => Ok(Success {
            lexer: lexer.into_consumed(),
            value: (),
        }),

        // Unexpected token.
        Some(Ok(_)) => Err(Failure {
            lexer,
            reason: Reason::UnexpectedToken,
            source: None,
        }),
    }
}

////////////////////////////////////////////////////////////////////////////////
// one
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which consumes a single token if it matches the given
/// token.
pub fn one<'t, Sc, Nl>(token: Sc::Token)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    move |mut lexer| {
        match lexer.next() {
            // Lexer error.
            Some(Err(e)) => Err(Failure {
                lexer,
                reason: Reason::LexerError,
                source: Some(Box::new(e)),
            }),

            // Matching token.
            Some(Ok(lex)) if lex == token => Ok(Success {
                lexer: lexer.into_consumed(),
                value: (),
            }),

            // Incorrect token.
            Some(Ok(_)) => Err(Failure {
                lexer,
                reason: Reason::UnexpectedToken,
                source: None,
            }),

            // Unexpected End-of-text.
            None => Err(Failure {
                lexer,
                reason: Reason::UnexpectedEndOfText,
                source: None,
            }),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// any
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the first which succeeds.
pub fn any<'t, Sc, Nl>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {
        for token in &tokens {
            match lexer.next() {
                // Lexer error.
                Some(Err(e)) => return Err(Failure {
                    lexer,
                    reason: Reason::LexerError,
                    source: Some(Box::new(e)),
                }),

                // Matching token.
                Some(Ok(lex)) if lex == *token => return Ok(Success {
                    lexer: lexer.into_consumed(),
                    value: (),
                }),

                // Incorrect token.
                // Unexpected End-of-text.
                _ => (),
            }
            lexer.reset();
        }

        Err(Failure {
            lexer,
            reason: Reason::UnexpectedToken,
            source: None,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// seq
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser attempts each of the given tokens in sequence, returning
/// the success only if each succeeds.
pub fn seq<'t, Sc, Nl>(tokens: &[Sc::Token])
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    let tokens = tokens.to_vec();
    move |mut lexer| {

        for token in &tokens {
            match lexer.next() {
                // Lexer error.
                Some(Err(e)) => return Err(Failure {
                    lexer,
                    reason: Reason::LexerError,
                    source: Some(Box::new(e)),
                }),

                // Matching token.
                Some(Ok(lex)) if lex == *token => (),

                // Incorrect token.
                Some(Ok(_)) => return Err(Failure {
                    lexer,
                    reason: Reason::UnexpectedToken,
                    source: None,
                }),

                // Unexpected End-of-text.
                None => return Err(Failure {
                    lexer,
                    reason: Reason::UnexpectedEndOfText,
                    source: None,
                }),
            }
        }

        Ok(Success {
            lexer: lexer.into_consumed(),
            value: (),
        })
    }
}
