////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Result for a failed parse.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::result::ParseError;
use crate::result::ParseErrorOwned;
use crate::span::NewLine;



////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with borrowed data.
pub struct Failure<'text, Sc, Nl> where Sc: Scanner {
    // TODO: Decide what lexer state to store on parse errors.
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, Sc, Nl>,
    /// The parse error.
    pub parse_error: ParseError<'text, Nl>,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text, Sc, Nl> Failure<'text, Sc, Nl> 
    where Sc: Scanner,
{
    #[cfg(test)]
    pub fn error_span_display(self) -> (&'static str, String) {
        (self.parse_error.description(), format!("{}", self.lexer.span()))
    }
}

impl<'text, Sc, Nl> std::fmt::Debug for Failure<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'text, Sc, Nl> std::fmt::Display for Failure<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parse_error)
    }
}

impl<'text, Sc, Nl> std::error::Error for Failure<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

#[cfg(test)]
impl<'text, Sc, Nl> PartialEq for Failure<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}


////////////////////////////////////////////////////////////////////////////////
// FailureOwned
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with owned data.
///
/// Similar to [`Failure`], except this  version owns all of its data, and can
/// thus  be used as an [`Error`] [`source`].
///
/// [`Failure`]: struct.Failure.html
/// [`Error`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
/// [`source`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html#method.source
#[derive(Debug)]
pub struct FailureOwned {
    /// The parse error.
    pub parse_error: ParseErrorOwned,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text, Sc, Nl> From<Failure<'text, Sc, Nl>> for FailureOwned
    where
        Sc: Scanner,
        Nl: NewLine,
{
    fn from(other: Failure<'text, Sc, Nl>) -> Self {
        FailureOwned {
            parse_error: other.parse_error.into_owned(),
            source: other.source,
        }
    }
}

impl std::fmt::Display for FailureOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parse_error)
    }
}

impl std::error::Error for FailureOwned {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

