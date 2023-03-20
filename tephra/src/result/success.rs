////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Result for a successful parse.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use tephra_span::Span;

// Standard library imports.
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////
// Success
////////////////////////////////////////////////////////////////////////////////
/// The result of a successful parse.
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Success<'text, Sc, V>
    where Sc: Scanner
{
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, Sc>,
    /// The parsed value.
    pub value: V,
}

impl<'text, Sc, V> Success<'text, Sc, V>
    where Sc: Scanner
{
    /// Constructs a new `Success` containing the given value and lexer state.
    pub fn new(value: V, lexer: Lexer<'text, Sc>) -> Self {
        Success {
            value,
            lexer,
        }
    }

    /// Consumes the Success and returns its parsed value.
    pub fn into_value(self) -> V {
        self.value
    }

    /// Converts `Success<'_, _, _, V>` into a `Success<'_, _, _, U>` by
    /// applying the given closure.
    pub fn map_value<F, U>(self, f: F) -> Success<'text, Sc, U> 
        where F: FnOnce(V) -> U
    {
        Success {
            value: (f)(self.value),
            lexer: self.lexer,
        }
    }

    /// Splits the Success into a tuple containing its parsed value and its
    /// other components.
    pub fn take_value(self) -> (V, Success<'text, Sc, ()>) {
        (self.value, Success {
            value: (),
            lexer: self.lexer,
        })
    }

    #[cfg(test)]
    pub fn value_span_display(self) -> (V, String) {
        (self.value, format!("{}", self.lexer.parse_span()))
    }
}

impl<'text, Sc, V> Debug for Success<'text, Sc, V>
    where
        Sc: Scanner,
        V: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Success")
            .field("value", &self.value)
            .field("lexer", &self.lexer)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Spanned
////////////////////////////////////////////////////////////////////////////////
/// A parsed value with its span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The span of the value's source text.
    pub span: Span,
    /// The parsed value.
    pub value: T,
}

impl<T> Spanned<T> {
    /// Converts `Spanned<'_, T>` into a `Spanned<'_, U>` by applying the given
    /// closure.
    pub fn map_value<F, U>(self, f: F) -> Spanned<U> 
        where F: FnOnce(T) -> U
    {
        Spanned {
            span: self.span,
            value: (f)(self.value),
        }
    }
}
