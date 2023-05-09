////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common lexer errors.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::error::SourceError;
use crate::error::SourceErrorRef;
use crate::ParseError;
use crate::SpanDisplay;

// External library imports.
use tephra_span::SourceTextRef;
use tephra_span::Span;

// Standard library imports.
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Write;
use std::iter::IntoIterator;


////////////////////////////////////////////////////////////////////////////////
// UnrecognizedTokenError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a token is unrecognized.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub struct UnrecognizedTokenError {
    /// The span of the parse up to the start of the unrecognized token.
    pub parse_span: Span,
}

impl UnrecognizedTokenError {
    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    #[must_use]
    pub fn into_source_error(self, source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceError::new(source_text, "unrecognized token")
            .with_span_display(SpanDisplay::new(
                source_text,
                Span::new_at(self.parse_span.end())))
            .with_cause(Box::new(self))
    }
}

impl Display for UnrecognizedTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unrecognized token {}",
            self.parse_span.end())
    }
}

impl Error for UnrecognizedTokenError {}

impl ParseError for UnrecognizedTokenError {
    fn parse_span(&self) -> Option<Span> {
        Some(self.parse_span)
    }
    
    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// Expected and found token descriptors.
////////////////////////////////////////////////////////////////////////////////
/// Describes expected parseable tokens.
#[derive(Debug, Clone)]
pub enum Expected<T> {
    /// Expected one token.
    Token(T),
    /// Expected one or more kinds of tokens.
    Tokens(Vec<T>),
    /// Expected end-of-text.
    EndOfText,
    /// Expected any token.
    AnyToken,
    /// Expected something else.
    Other(String),
}

impl<T> Expected<T> {
    /// The maximum number of items to display in the expected items
    /// description.
    const MAX_EXPECTED_DISPLAY: usize = 5;

    /// Returns `true` if the expected token list is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Tokens(toks) if toks.is_empty())
    }

    /// Constructs an `Expected` description for any of the given tokens.
    pub fn any<I>(tokens: I) -> Self
        where I: IntoIterator<Item=T>
    {
        Self::Tokens(tokens.into_iter().collect())
    }
}

impl<T> Display for Expected<T> where T: Debug + Display + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expected::*;

        match self {
            Token(tok)                      => write!(f, "{tok}"),
            Tokens(toks) if toks.is_empty() => write!(f, "nothing"),
            Tokens(toks) if toks.len() == 1 => write!(f, "{}", toks[0]),
            
            Tokens(toks) if toks.len() < Self::MAX_EXPECTED_DISPLAY
                => write!(f, "one of {}", fmt_list(&toks[..])?),
            
            Tokens(toks) => write!(f,
                "one of {}",
                fmt_list(&toks[..Self::MAX_EXPECTED_DISPLAY])?),

            Other(message) => write!(f, "{message}"),
            EndOfText      => write!(f, "end of text"),
            AnyToken       => write!(f, "any token"),
        }
    }
}

/// Formats an iterator of items into a comma-delimited list.
fn fmt_list<I, T>(items: I) -> Result<String, std::fmt::Error>
    where
        I: IntoIterator<Item=T>,
        T: Display,
{
    let mut items = items.into_iter();
    let mut res = String::new();

    if let Some(item) = items.next() {
        write!(&mut res, "{item}")?;
    }
    for item in items {
        write!(&mut res, ", {item}")?;
    }
    Ok(res)
}


/// Describes found parseable tokens.
#[derive(Debug, Clone)]
pub enum Found<T> {
    /// Found the given token.
    Token(T),
    /// Found the end-of-text.
    EndOfText,
}

impl<T> Display for Found<T> where T: Debug + Display + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Found::*;

        match self {
            Token(tok) => write!(f, "{tok}"),
            EndOfText => write!(f, "end of text"),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// UnexpectedTokenError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a token is unexpected.
#[derive(Debug, Clone)]
pub struct UnexpectedTokenError<T>
    where T: Debug + Display + Send + Sync + 'static
{
    /// The span of the parse up to the start of the unexpected token.
    pub parse_span: Span,
    /// The span of the found token.
    pub token_span: Span,
    /// The expected tokens.
    pub expected: Expected<T>,
    /// The found token.
    pub found: Found<T>,
}

impl<T> UnexpectedTokenError<T> where T: Debug + Display + Send + Sync {
    /// Constructs a string describing the expected and found tokens.
    fn expected_description(&self) -> String {
        if self.expected.is_empty() {
            format!("found {}", self.found)
        } else {
            format!("expected {}; found {}", self.expected, self.found)
        }
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    pub fn into_source_error(self, source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceError::new(source_text, "unexpected token")
            .with_span_display(SpanDisplay::new_error_highlight(
                source_text,
                self.token_span,
                self.expected_description()))
            .with_cause(Box::new(self))
    }
}

impl<T> Display for UnexpectedTokenError<T>
    where T: Debug + Display + Send + Sync + 'static
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}",
            self.expected_description(),
            self.token_span)
    }
}

impl<T> Error for UnexpectedTokenError<T>
    where T: Debug + Display + Send + Sync + 'static {}

impl<T> ParseError for UnexpectedTokenError<T>
    where T: Debug + Display + Send + Sync + 'static
{
    fn parse_span(&self) -> Option<Span> {
        Some(self.parse_span)
    }
    
    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// RecoverError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when error recovery fails.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub struct RecoverError;

impl Display for RecoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to recover from previous error")
    }
}

impl Error for RecoverError {}

impl ParseError for RecoverError {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}
