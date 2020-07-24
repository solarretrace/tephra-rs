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
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseResult;
use crate::result::Success;
use crate::result::Failure;
use crate::result::Reason;


////////////////////////////////////////////////////////////////////////////////
// 
////////////////////////////////////////////////////////////////////////////////

/// Parses the end of the text.
pub fn end_of_text<'t, F, S, V>(mut lexer: Lexer<'t, S>)
    -> ParseResult<'t, S, ()>
    where S: Scanner,
{
    let saved = lexer.clone();
    match lexer.next() {
        // Unexpected End-of-text
        None => Ok(Success {
            span: lexer.current_span(),
            lexer: lexer,
            value: (),
        }),

        Some(Ok(lex)) => Err(Failure {
            span: *lex.span(),
            lexer: saved,
            reason: Reason::UnexpectedToken,
            source: None,
        }),

        // Lexer error.
        Some(Err(e)) => Err(Failure {
            span: lexer.current_span(),
            lexer: saved,
            reason: Reason::LexerError,
            source: Some(Box::new(e)),
        }),
    }
}

/// Returns a parser which consumes a single token if it matches the given
/// token.
pub fn one<'t, S>(token: S::Token)
    -> impl FnMut(Lexer<'t, S>) -> ParseResult<'t, S, ()>
    where S: Scanner
{
    move |mut lexer| {
        let saved = lexer.clone();
        match lexer.next() {
            // Correct token.
            Some(Ok(lex)) if lex == token => Ok(Success {
                lexer: lexer,
                span: lex.into_span(),
                value: (),
            }),

            // Incorrect token.
            Some(Ok(lex)) => Err(Failure {
                span: *lex.span(),
                lexer: saved,
                reason: Reason::UnexpectedToken,
                source: None,
            }),

            // Lexer error.
            Some(Err(e)) => Err(Failure {
                span: lexer.current_span(),
                lexer: saved,
                reason: Reason::LexerError,
                source: Some(Box::new(e)),
            }),

            // Unexpected End-of-text
            None => Err(Failure {
                span: lexer.current_span(),
                lexer: saved,
                reason: Reason::UnexpectedEndOfText,
                source: None,
            }),
        }
    }
}
