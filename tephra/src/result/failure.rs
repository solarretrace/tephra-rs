////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
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



////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with borrowed data.
pub struct Failure<'text, Sc> where Sc: Scanner {
    // TODO: Decide what lexer state to store on parse errors.
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, Sc>,
    /// The parse error.
    pub parse_error: ParseError<'text>,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text, Sc> Failure<'text, Sc> 
    where Sc: Scanner,
{
    /// Attempts to push a `ParseError` into the `Failure` as contextual
    /// information.
    pub fn push_context(mut self, error: ParseError<'text>) -> Self {
        if error.section_type() > self.parse_error.section_type() {
            // Replace current parse error.
            self.parse_error = error;
            self
        } else {
            // Convert current Failure to FailureOwned and push it into source.
            let parse_error = std::mem::replace(&mut self.parse_error, error)
                .into();
            let source = self.source.take();
            self.source = Some(Box::new(FailureOwned { parse_error, source }));
            self
        }
    }

    #[cfg(test)]
    pub fn error_span_display(self) -> (&'static str, String) {
        (self.parse_error.description(), format!("{}", self.lexer.parse_span()))
    }
}

impl<'text, Sc> std::fmt::Debug for Failure<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'text, Sc> std::fmt::Display for Failure<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parse_error)?;
        if let Some(src) = &self.source {
            write!(f, "Caused by {}", src)?;
        }
        Ok(())
    }
}

impl<'text, Sc> std::error::Error for Failure<'text, Sc>
    where Sc: Scanner,
{
    fn description(&self) -> &str {
        &self.parse_error.description()
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

#[cfg(test)]
impl<'text, Sc> PartialEq for Failure<'text, Sc>
    where Sc: Scanner,
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

impl<'text, Sc> From<Failure<'text, Sc>> for FailureOwned
    where Sc: Scanner,
{
    fn from(other: Failure<'text, Sc>) -> Self {
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
    fn description(&self) -> &str {
        &self.parse_error.description()
    }
    
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

