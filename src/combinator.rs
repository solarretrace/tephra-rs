////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::result::ParseResult;
use crate::result::Success;


////////////////////////////////////////////////////////////////////////////////
// Parse result substitution combinators
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'t, F, K, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, K>) -> ParseResult<'t, K, ()>
    where F: FnMut(Lexer<'t, K>) -> ParseResult<'t, K, V>,
{
    move |lx| {
        match (parser)(lx) {
            Ok(pass) => {
                Ok(Success {
                    lexer: pass.lexer,
                    span: pass.span,
                    value: (),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}

/// A combinator which replaces a parsed value with the source text of the
/// parsed span.
pub fn text<'t, F, K, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, K>) -> ParseResult<'t, K, &'t str>
    where F: FnMut(Lexer<'t, K>) -> ParseResult<'t, K, V>,
{
    move |lx| {
        match (parser)(lx) {
            Ok(pass) => {
                let value = pass.span.text();
                Ok(Success {
                    lexer: pass.lexer,
                    span: pass.span,
                    value,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}
