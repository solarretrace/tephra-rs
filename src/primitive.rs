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
use crate::lexer::Tokenize;
use crate::result::ParseResult;
use crate::result::Success;
use crate::result::Failure;


////////////////////////////////////////////////////////////////////////////////
// 
////////////////////////////////////////////////////////////////////////////////



pub fn one<'t, F, K, V>(token: K::Token)
    -> impl FnMut(Lexer<'t, K>) -> ParseResult<'t, K, ()>
    where K: Tokenize
{
    move |mut lx| {
        match lx.next() {
            // Correct token.
            Some(Ok(lex)) if lex == token => Ok(Success {
                lexer: lx,
                span: lex.into_span(),
                value: (),
            }),

            // Incorrect token.
            Some(Ok(lex)) => Err(Failure {
                span: *lex.span(),
                lexer: lx,
                source: None,
                context: None,
            }),

            // Lexer error.
            Some(Err(e)) => Err(Failure {
                span: lx.current_span(),
                lexer: lx,
                source: Some(Box::new(e)),
                context: None,
            }),

            // Unexpected End-of-text
            None => Err(Failure {
                span: lx.current_span(),
                lexer: lx,
                source: None,
                context: None,
            }),
        }
    }
}
