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
use crate::Success;

// External library imports.
use tephra_error::ParseError;
use tephra_tracing::Level;
use tephra_tracing::event;



////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, Sc, V> 
        = Result<Success<'text, Sc, V>, Box<dyn ParseError<'text>>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt<'text, Sc, V> 
    where Sc: Scanner,
{
    /// Converts the ParseResult into a Result containing the parsed value,
    /// discarding any associated spans or lexer state.
    fn finish(self)
        -> Result<V, Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// Converts `ParseResult<'_, _, _, V>` into a `ParseResult<'_, _, _, U>` by
    /// applying the given closure.
    fn map_value<F, U>(self, f: F) -> ParseResult<'text, Sc, U>
        where F: FnOnce(V) -> U;

    /// Applies the given function to the `ParseResult`'s lexer.
    fn map_lexer<F>(self, f: F) -> ParseResult<'text, Sc, V>
        where F: FnOnce(Lexer<'text, Sc>) -> Lexer<'text, Sc>;

    /// Applies any `ErrorTransform`s in the given `Context`.
    fn apply_context(self, ctx: Context<'text, Sc>) -> Self;

    /// Outputs a trace event displaying the parse result.
    fn trace_result(self, level: Level, label: &'static str) -> Self;
}


impl<'text, Sc, V> ParseResultExt<'text, Sc, V> for ParseResult<'text, Sc, V>
    where Sc: Scanner,
{
    fn finish(self)
        -> Result<V, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        self
            .map(Success::into_value)
            .map_err(|e| e.into_owned())
    }

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

    #[cfg_attr(not(feature="tracing"),
        allow(unused_variables),
        allow(unused_results))]
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
                    => event!(Level::TRACE, "{} Err\n{}", label, fail),
            }
        };

        self
    }
}

