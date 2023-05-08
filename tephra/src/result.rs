////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse result trait and type.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Context;
use crate::lexer::Lexer;
use crate::lexer::Scanner;

// External library imports.
use tephra_error::ParseError;
use tephra_span::Span;

// Standard library imports.
use std::fmt::Debug;


////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, Sc, V> 
        = Result<Success<'text, Sc, V>, Box<dyn ParseError>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, Sc, V> 
    where Sc: Scanner,
{
    /// Converts `ParseResult<'_, _, _, V>` into a `ParseResult<'_, _, _, U>` by
    /// applying the given closure.
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, U>
        where F: FnOnce(V) -> U;

    /// Applies the given function to the `ParseResult`'s lexer.
    fn map_lexer<F>(self, f: F) -> ParseResult<'text, Sc, V>
        where F: FnOnce(Lexer<'text, Sc>) -> Lexer<'text, Sc>;

    /// Applies any `ErrorTransform`s in the given `Context`.
    fn apply_context(self, ctx: Context<'text, Sc>) -> Self;
}


impl<'text, Sc, V> ParseResultExt<'text, Sc, V> for ParseResult<'text, Sc, V>
    where Sc: Scanner,
{
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, U> 
        where F: FnOnce(V) -> U,
    {
        match self {
            Ok(succ)  => Ok(succ.map_value(f)),
            Err(fail) => Err(fail),
        }
    }

    fn map_lexer<F>(self, f: F) -> ParseResult<'text, Sc, V>
        where F: FnOnce(Lexer<'text, Sc>) -> Lexer<'text, Sc>
    {
        match self {
            Ok(Success { value, lexer })  => Ok(Success {
                value,
                lexer: (f)(lexer),
            }),
            Err(fail) => Err(fail),
        }
    }

    fn apply_context(self, ctx: Context<'text, Sc>) -> Self {
        match self {
            Ok(succ)  => Ok(succ),
            Err(fail) => Err(ctx.apply_error_transform_recursive(fail)),
        }
    }
}



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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
