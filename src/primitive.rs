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
use crate::span::Span;
use crate::span::NewLine;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseResult;
use crate::result::Success;
use crate::result::Failure;
use crate::result::Reason;


////////////////////////////////////////////////////////////////////////////////
// end-of-text
////////////////////////////////////////////////////////////////////////////////
/// Parses the end of the text.
pub fn end_of_text<'t, F, Sc, Nl, V>(mut lexer: Lexer<'t, Sc, Nl>)
    -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    let saved = lexer.clone();
    match lexer.next() {
        // Lexer error.
        Some(Err(e)) => Err(Failure {
            span: lexer.current_span(),
            lexer: saved,
            reason: Reason::LexerError,
            source: Some(Box::new(e)),
        }),

        // Expected End-of-text.
        None => Ok(Success {
            span: lexer.current_span(),
            lexer: lexer,
            value: (),
        }),

        // Unexpected token.
        Some(Ok(lex)) => Err(Failure {
            span: *lex.span(),
            lexer: saved,
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
        let saved = lexer.clone();
        match lexer.next() {
            // Lexer error.
            Some(Err(e)) => Err(Failure {
                span: lexer.current_span(),
                lexer: saved,
                reason: Reason::LexerError,
                source: Some(Box::new(e)),
            }),

            // Matching token.
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

            // Unexpected End-of-text.
            None => Err(Failure {
                span: lexer.current_span(),
                lexer: saved,
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
                    span: lexer.current_span(),
                    lexer,
                    reason: Reason::LexerError,
                    source: Some(Box::new(e)),
                }),

                // Matching token.
                Some(Ok(lex)) if lex == *token => return Ok(Success {
                    lexer,
                    span: lex.into_span(),
                    value: (),
                }),

                // Incorrect token.
                // Unexpected End-of-text.
                _ => (),
            }
            lexer.reset();
        }

        Err(Failure {
            span: lexer.current_span(),
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
        let start = lexer.current_pos();
        let mut end = lexer.current_pos();

        for token in &tokens {
            match lexer.next() {
                // Lexer error.
                Some(Err(e)) => return Err(Failure {
                    span: lexer.current_span(),
                    lexer,
                    reason: Reason::LexerError,
                    source: Some(Box::new(e)),
                }),

                // Matching token.
                Some(Ok(lex)) if lex == *token => {
                    end = lex.span().end();
                },

                // Incorrect token.
                Some(Ok(lex)) => return Err(Failure {
                    span: *lex.span(),
                    lexer,
                    reason: Reason::UnexpectedToken,
                    source: None,
                }),

                // Unexpected End-of-text.
                None => return Err(Failure {
                    span: lexer.current_span(),
                    lexer,
                    reason: Reason::UnexpectedEndOfText,
                    source: None,
                }),
            }
        }

        Ok(Success {
            span: Span::new_enclosing(start, end, lexer.source()),
            lexer,
            value: (),
        })
    }
}
