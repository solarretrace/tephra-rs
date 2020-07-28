////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse results.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod success;
mod failure;
mod display;

// Exports.
pub use self::success::*;
pub use self::failure::*;
pub use self::display::*;

use crate::lexer::Scanner;

////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, Sc, Nl, V> 
        = Result<Success<'text, Sc, Nl, V>, Failure<'text, Sc, Nl>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, Sc, Nl, V> 
    where Sc: Scanner,
{
    /// Converts the ParseResult into a Result containing the parsed value,
    /// discarding any associated spans or lexer state.
    fn finish(self) -> Result<V, FailureOwned>;

    /// Converts ParseResult<'_, _, _, V> into a ParseResult<'_, _, _, U> by
    /// applying the given closure.
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, Nl, U> 
        where F: FnOnce(V) -> U;

    /// Consumes the current span on the Success's contained lexer.
    fn consume_current_if_success(self) -> Self;
}

impl<'text, Sc, Nl, V> ParseResultExt<'text, Sc, Nl, V>
        for ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
{
    fn finish(self) -> Result<V, FailureOwned> {
        self
            .map(Success::into_value)
            .map_err(FailureOwned::from)
    }

    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, Nl, U> 
        where F: FnOnce(V) -> U,
    {
        match self {
            Ok(succ)  => Ok(succ.map_value(f)),
            Err(fail) => Err(fail),
        }
    }

    fn consume_current_if_success(self) -> Self {
        match self {
            Ok(succ)  => Ok(succ.consumed_current()),
            Err(fail) => Err(fail),
        }
    }
}

