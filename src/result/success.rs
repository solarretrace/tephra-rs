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
use crate::position::ColumnMetrics;
use crate::span::Span;


////////////////////////////////////////////////////////////////////////////////
// Success
////////////////////////////////////////////////////////////////////////////////
/// The result of a successful parse.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Success<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, Sc, Cm>,
    /// The parsed value.
    pub value: V,
}

impl<'text, Sc, Cm, V> Success<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    /// Consumes the Success and returns its parsed value.
    pub fn into_value(self) -> V {
        self.value
    }

    /// Converts Success<'_, _, _, V> into a Success<'_, _, _, U> by
    /// applying the given closure.
    pub fn map_value<F, U>(self, f: F) -> Success<'text, Sc, Cm, U> 
        where F: FnOnce(V) -> U
    {
        Success {
            lexer: self.lexer,
            value: (f)(self.value),
        }
    }

    /// Splits the Success into a tuple containing its parsed value and its
    /// other components.
    pub fn take_value(self) -> (V, Success<'text, Sc, Cm, ()>) {
        (self.value, Success {
            lexer: self.lexer,
            value: (),
        })
    }

    #[cfg(test)]
    pub fn value_span_display(self) -> (V, String) {
        (self.value, format!("{}", self.lexer.span()))
    }
}


////////////////////////////////////////////////////////////////////////////////
// Spanned
////////////////////////////////////////////////////////////////////////////////
/// A parsed value with its span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<'text, T> {
    /// The parsed value.
    pub value: T,
    /// The span of the value's source text.
    pub span: Span<'text>,
}
