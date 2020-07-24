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
pub type ParseResult<'text, K, V> 
        = Result<Success<'text, K, V>, Failure<'text, K>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, K, V> 
    where K: Scanner,
{
    /// Converts the ParseResult into a Result containing the parsed value,
    /// discarding any associated spans or lexer state.
    fn finish(self) -> Result<V, FailureOwned>;
}

impl<'text, K, V> ParseResultExt<'text, K, V> for ParseResult<'text, K, V>
    where K: Scanner,
{
    fn finish(self) -> Result<V, FailureOwned> {
        self
            .map(Success::into_value)
            .map_err(FailureOwned::from)
    }
}

