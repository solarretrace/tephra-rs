////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Result for a failed parse.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use tephra_error::ParseError;



////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with borrowed data.
#[derive(Debug)]
pub struct Failure<'text, Sc> where Sc: Scanner {
    // TODO: Decide what lexer state to store on parse errors.
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, Sc>,
    /// The parse error.
    pub parse_error: ParseError<'text>,
}

impl<'text, Sc> Failure<'text, Sc> 
    where Sc: Scanner,
{
    /// Constructs a new Failure containing the given parse error and lexer
    /// state.
    pub fn new(parse_error: ParseError<'text>, lexer: Lexer<'text, Sc>) -> Self
    {
        Failure {
            lexer,
            parse_error,
        }
    }

    /// Applies the given `ParseError` as a contextual wrapper around the
    /// contained error.
    pub fn with_context(mut self, context: ParseError<'text>) -> Self {
        self.push_context(context);
        self
    }

    /// Applies the given `ParseError` as a contextual wrapper around the
    /// contained error.
    pub fn push_context(&mut self, context: ParseError<'text>) {
        self.parse_error.push_error_context(context)
    }

    #[cfg(test)]
    pub fn error_span_display(&self) -> (&str, String) {
        (self.parse_error.description(), format!("{}", self.lexer.parse_span()))
    }
}

impl<'text, Sc> std::fmt::Display for Failure<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parse_error)
    }
}

impl<'text, Sc> std::error::Error for Failure<'text, Sc>
    where Sc: Scanner,
{
    fn description(&self) -> &str {
        &self.parse_error.description()
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.parse_error.source()
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

