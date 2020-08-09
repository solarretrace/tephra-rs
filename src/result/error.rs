////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse error.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::span::Span;
use crate::span::NewLine;
use crate::result::SourceDisplay;
use crate::result::SourceSpan;


////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseError<'text, Nl> {
    description: &'static str,
    span: Option<(Span<'text, Nl>, String)>,
    is_lexer_error: bool,
}

impl<'text, Nl> ParseError<'text, Nl> {
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            is_lexer_error: false,
        }
    }

    pub fn with_span<S>(mut self, message: S, span: Span<'text, Nl>) -> Self 
        where S: Into<String>,
    {
        self.span = Some((span, message.into()));
        self
    }

    pub fn unrecognized_token(span: Span<'text, Nl>) -> Self {
        ParseError {
            description: "unrecognized token",
            span: Some((span, "symbol not recognized".to_owned())),
            is_lexer_error: true,
        }
    }

    pub fn unexpected_token<T>(span: Span<'text, Nl>, expected: T) -> Self 
        where T: std::fmt::Display
    {
        ParseError {
            description: "unexpected token",
            span: Some((span, format!("expected {}", expected))),
            is_lexer_error: true,
        }
    }

    pub fn unexpected_end_of_text(span: Span<'text, Nl>) -> Self {
        ParseError {
            description: "unexpected end of text",
            span: Some((span, "text ends here".to_owned())),
            is_lexer_error: true,
        }
    }

    pub fn unexpected_text(span: Span<'text, Nl>) -> Self {
        ParseError {
            description: "expected end of text",
            span: Some((span, "text should end here".to_owned())),
            is_lexer_error: true,
        }
    }

    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn is_lexer_error(&self) -> bool {
        self.is_lexer_error
    }
}

impl<'text, Nl> ParseError<'text, Nl> 
    where Nl: NewLine,
{
    pub fn into_owned(self) -> ParseErrorOwned 
    {
        ParseErrorOwned {
            display: format!("{}", self),
            is_lexer_error: self.is_lexer_error,
        }
    }
}


impl<'text, Nl> Default for ParseError<'text, Nl>
    where Nl: NewLine,
{
    fn default() -> Self {
        ParseError {
            description: "parse error",
            span: None,
            is_lexer_error: false,
        }
    }
}

impl<'text, Nl> std::fmt::Display for ParseError<'text, Nl>
    where Nl: NewLine,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((span, msg)) = &self.span {
            let span = *span;
            let msg = &*msg;
            write!(f, "{}", 
                SourceDisplay::new(self.description)
                    .with_error_type()
                    .with_source_span(
                        SourceSpan::new_error_highlight(span, msg)))
        } else {
            // TODO: Clean up message.
            write!(f, "{} NO SPAN", self.description)
        }

    }
}

impl<'text, Nl> From<&'static str> for ParseError<'text, Nl> {
    fn from(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            is_lexer_error: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ParseErrorOwned
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseErrorOwned {
    display: String,
    is_lexer_error: bool,
}


impl std::fmt::Display for ParseErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}
