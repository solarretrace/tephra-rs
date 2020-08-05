////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod join;
mod option;
mod primitive;
mod repeat;

// Exports.
pub use join::*;
pub use option::*;
pub use primitive::*;
pub use repeat::*;


// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::Success;
use crate::combinator::empty;


////////////////////////////////////////////////////////////////////////////////
// Control combinators.
////////////////////////////////////////////////////////////////////////////////
/// A combinator which disables all token filters during exectution of the given
/// parser.
pub fn exact<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |mut lexer| {
        let filter = lexer.take_filter();
        match (parser)(lexer) {
            Ok(mut succ)  => {
                succ.lexer.set_filter(filter);
                Ok(succ)
            },
            Err(mut fail) => {
                fail.lexer.set_filter(filter);
                Err(fail)
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parse result substitution combinators.
////////////////////////////////////////////////////////////////////////////////

/// A combinator which discards a parsed value, replacing it with `()`.
pub fn discard<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, ()>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        match (parser)(lexer) {
            Ok(succ) => {
                Ok(Success {
                    lexer: succ.lexer,
                    value: (),
                })
            },
            Err(fail) => Err(fail),
        }
    }
}

/// A combinator which replaces a parsed value with the source text of the
/// parsed span.
pub fn text<'t, Sc, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, &'t str>
    where
        Sc: Scanner,
        Nl: NewLine,
        F: FnMut(Lexer<'t, Sc, Nl>) -> ParseResult<'t, Sc, Nl, V>,
{
    move |lexer| {
        let start = lexer.current_pos().byte;
        match (parser)(lexer) {
            Ok(succ) => {
                let end = succ.lexer.current_pos().byte;
                let value = &succ.lexer.source()[start..end];

                Ok(Success {
                    lexer: succ.lexer,
                    value,
                })
            },
            Err(fail) => Err(fail),
        }
    }
}


