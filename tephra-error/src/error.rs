////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse error.
////////////////////////////////////////////////////////////////////////////////


// Internal modules.
mod delimit;
mod external;
mod lexer;
mod source;

// Exports.
pub use delimit::*;
pub use lexer::*;
pub use source::*;

// External library imports.
use tephra_span::SourceTextRef;
use tephra_span::Span;


////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////

/// Provides common methods for parse errors.
pub trait ParseError: std::error::Error + Send + Sync 
    + AsError<dyn std::error::Error + Send + Sync>
{
    /// Returns the span of the current parse up to the start of the error, if
    /// available.
    fn error_span(&self) -> Option<Span> { None }

    /// Returns `true` if the error type is recoverable.
    fn is_recoverable(&self) -> bool { true }

    /// Converts a `ParseError` into a `SourceErrorRef<'text>`.
    #[must_use]
    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceError::new(source_text, format!("{self}"))
            .with_cause(self.into_error())
    }

    /// Converts a `ParseError` into an owned error.
    fn into_error(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>;
}


impl<E> ParseError for Box<E>
    where E: std::error::Error + Send + Sync + 'static
{
    fn into_error(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>
    {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// Error casting traits
////////////////////////////////////////////////////////////////////////////////
pub trait AsErrorFrom<T>
    where T: ?Sized + std::error::Error + Send + Sync
{
    fn as_error_from(value: &T) -> &Self;
    fn as_error_from_mut(value: &mut T) -> &mut Self;
}

impl<'a, T> AsErrorFrom<T> for dyn std::error::Error + Send + Sync + 'a
    where T: std::error::Error + Send + Sync + 'a
{
    fn as_error_from(value: &T) -> &(dyn std::error::Error + Send + Sync + 'a) {
        value
    }
    fn as_error_from_mut(value: &mut T)
        -> &mut (dyn std::error::Error + Send + Sync + 'a)
    {
        value
    }
}

/// Provides methods to upcast trait objects into `std::error::Error`.
pub trait AsError<E: ?Sized> {
    fn as_error(&self) -> &E;
    fn as_error_mut(&mut self) -> &mut E;
}

impl<'a, T> AsError<dyn std::error::Error + Send + Sync + 'a> for T
    where T: std::error::Error + Send + Sync + 'a
{
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'a) {
        AsErrorFrom::as_error_from(self)
    }
    
    fn as_error_mut(&mut self)
        -> &mut (dyn std::error::Error + Send + Sync + 'a)
    {
        AsErrorFrom::as_error_from_mut(self)
    }
}
