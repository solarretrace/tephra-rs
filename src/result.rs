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
mod error;

// Exports.
pub use self::success::*;
pub use self::failure::*;
pub use self::display::*;
pub use self::error::*;

use crate::lexer::Scanner;
use crate::span::NewLine;

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

    // /// Converts ParseResult<'_, _, _, V> into a ParseResult<'_, _, _, U> by
    // /// applying the given closure. If the closure return an Err, the result
    // /// will become an error as well.
    // fn convert_value<F, U, P, E>(self, f: F) -> ParseResult<'text, Sc, Nl, U> 
    //     where
    //         F: FnOnce(V) -> Result<U, (P, Option<E>)>,
    //         P: Into<ParseError<'text, Nl>>,
    //         E: std::error::Error + Send + Sync + 'static;

}

impl<'text, Sc, Nl, V> ParseResultExt<'text, Sc, Nl, V>
        for ParseResult<'text, Sc, Nl, V>
    where
        Sc: Scanner,
        Nl: NewLine,
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

    // fn convert_value<F, U, P, E>(self, f: F) -> ParseResult<'text, Sc, Nl, U> 
    //     where
    //         F: FnOnce(V) -> Result<U, (P, Option<E>)>,
    //         P: Into<ParseError<'text, Nl>>,
    //         E: std::error::Error + Send + Sync + 'static,
    // {
    //     match self {
    //         Ok(succ) => {
    //             let (v, succ) = succ.take_value();
    //             match (f)(v) {
    //                 Ok(value) => Ok(succ.map_value(|_| value)),
    //                 Err((parse_error, e)) => {
    //                     let parse_error = parse_error.into();
    //                     let source = match e {
    //                         None => None,
    //                         Some(e) => Some(Box::new(e)),
    //                     };
    //                     Err(Failure {
    //                         lexer: succ.lexer,
    //                         parse_error,
    //                         source: Some(Box::new(e)),
    //                     })
    //                 },
    //             }
    //         },
    //         Err(fail) => Err(fail),
    //     }
    // }
}

