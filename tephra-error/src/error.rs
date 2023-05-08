////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse error.
////////////////////////////////////////////////////////////////////////////////


mod delimit;
mod external;
mod lexer;
mod source;

pub use delimit::*;
pub use external::*;
pub use lexer::*;
pub use source::*;


// External library imports.
use tephra_span::Span;
use tephra_span::SourceTextRef;



////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////

/// Provides common methods for parse errors.
pub trait ParseError: std::error::Error + Send + Sync 
    + AsError<dyn std::error::Error + Send + Sync>
{
    /// Returns the span of the current parse when the failure occurred, if
    /// available.
    fn parse_span(&self) -> Option<Span> { None }

    /// Converts a `ParseError` into a `SourceErrorRef<'text>`.
    fn into_source_error<'text>(
        self: Box<Self>,
        source_text: SourceTextRef<'text>)
        -> SourceErrorRef<'text>
    {
        SourceError::new(source_text, format!("{self}"))
            .with_cause(self.into_owned())
    }

    /// Converts a `ParseError` into an owned error.
    fn into_owned(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>;
}


impl<E> ParseError for Box<E>
    where E: std::error::Error + Send + Sync + 'static
{
    fn into_owned(self: Box<Self>)
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
