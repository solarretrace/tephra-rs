////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Result for a successful parse.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::span::Span;


////////////////////////////////////////////////////////////////////////////////
// Success
////////////////////////////////////////////////////////////////////////////////
/// The result of a successful parse.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Success<'text, S, Nl, V> where S: Scanner {
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, S, Nl>,
    /// The span of the parse result.
    pub span: Span<'text, Nl>,
    /// The parsed value.
    pub value: V,
}

impl<'text, S, Nl, V> Success<'text, S, Nl, V> where S: Scanner {
    /// Consumes the Success and returns its parsed value.
    pub fn into_value(self) -> V {
        self.value
    }
}
