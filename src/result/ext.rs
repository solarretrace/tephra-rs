////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse result trait and type.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::result::Success;
use crate::result::Failure;
use crate::result::FailureOwned;


////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, Sc, Cm, V> 
        = Result<Success<'text, Sc, Cm, V>, Failure<'text, Sc, Cm>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, Sc, Cm, V> 
    where Sc: Scanner,
{
    /// Converts the ParseResult into a Result containing the parsed value,
    /// discarding any associated spans or lexer state.
    fn finish(self) -> Result<V, FailureOwned>;

    /// Converts ParseResult<'_, _, _, V> into a ParseResult<'_, _, _, U> by
    /// applying the given closure.
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, Cm, U> 
        where F: FnOnce(V) -> U;

    /// Converts a ParseResult into a Result with an Option for its Err variant,
    /// which will be None if the failure is a lexer error.
    fn filter_lexer_error(self)
        -> Result<Success<'text, Sc, Cm, V>, Option<Failure<'text, Sc, Cm>>>;
}

impl<'text, Sc, Cm, V> ParseResultExt<'text, Sc, Cm, V>
        for ParseResult<'text, Sc, Cm, V>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    fn finish(self) -> Result<V, FailureOwned> {
        self
            .map(Success::into_value)
            .map_err(FailureOwned::from)
    }

    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, Cm, U> 
        where F: FnOnce(V) -> U,
    {
        match self {
            Ok(succ)  => Ok(succ.map_value(f)),
            Err(fail) => Err(fail),
        }
    }

    fn filter_lexer_error(self)
        -> Result<Success<'text, Sc, Cm, V>, Option<Failure<'text, Sc, Cm>>>
    {
        self.map_err(|e| if e.parse_error.is_lexer_error() {
                None
            } else {
                Some(e)
            })
    }
}

