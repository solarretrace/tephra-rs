////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse result trait and type.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Scanner;
use crate::result::Success;
use crate::result::Failure;
use crate::result::FailureOwned;

// External library imports.
use tracing::Level;
use tracing::event;

////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, Sc, V> 
        = Result<Success<'text, Sc, V>, Failure<'text, Sc>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, Sc, V> 
    where Sc: Scanner,
{
    /// Converts the ParseResult into a Result containing the parsed value,
    /// discarding any associated spans or lexer state.
    fn finish(self) -> Result<V, FailureOwned>;

    /// Converts `ParseResult<'_, _, _, V>` into a `ParseResult<'_, _, _, U>` by
    /// applying the given closure.
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, U> 
        where F: FnOnce(V) -> U;

    /// Outputs a trace event displaying the parse result.
    fn trace_result(self, level: Level, label: &'static str) -> Self;
}

impl<'text, Sc, V> ParseResultExt<'text, Sc, V>
        for ParseResult<'text, Sc, V>
    where Sc: Scanner,
{
    fn finish(self) -> Result<V, FailureOwned> {
        self
            .map(Success::into_value)
            .map_err(FailureOwned::from)
    }

    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, U> 
        where F: FnOnce(V) -> U,
    {
        match self {
            Ok(succ)  => Ok(succ.map_value(f)),
            Err(fail) => Err(fail),
        }
    }

    fn trace_result(self, level: Level, label: &'static str) -> Self {
        match level {
            Level::ERROR => event!(Level::ERROR,
                "{} {}",
                label,
                if self.is_ok() { "Ok" } else { "Err" }),
            
            Level::WARN => event!(Level::WARN,
                "{} {}",
                label,
                if self.is_ok() { "Ok" } else { "Err" }),
            
            Level::INFO => event!(Level::INFO,
                "{} {}",
                label,
                if self.is_ok() { "Ok" } else { "Err" }),
            
            Level::DEBUG => event!(Level::DEBUG,
                "{} {}",
                label,
                if self.is_ok() { "Ok" } else { "Err" }),

            Level::TRACE => match self.as_ref() {
                Ok(succ)
                    => event!(Level::TRACE, "{} Ok\n{}", label, succ.lexer),
                Err(fail)
                    => event!(Level::TRACE, "{} Err\n{}", label, fail.lexer),
            }
        };

        self
    }
}

