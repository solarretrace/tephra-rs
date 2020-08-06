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
use crate::result::SourceSpan;
use crate::result::Highlight;


////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseError<'text, Nl> {
    description: &'static str,
    span: Option<(Span<'text, Nl>, String)>,

}

impl<'text, Nl> ParseError<'text, Nl> {
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
        }
    }

    pub fn unrecognized_token(span: Span<'text, Nl>) -> Self {
        ParseError::new("unrecognized token")
            .with_span("symbol not recognized", span)
    }

    pub fn unexpected_token(span: Span<'text, Nl>) -> Self {
        ParseError::new("unexpected token")
            .with_span("token not expected", span)
    }

    pub fn unexpected_end_of_text(span: Span<'text, Nl>) -> Self {
        ParseError::new("unexpected end of text")
            .with_span("text ends here", span)
    }

    pub fn unexpected_text(span: Span<'text, Nl>) -> Self {
        ParseError::new("expected end of text")
            .with_span("text should end here", span)
    }

    pub fn with_span<S>(mut self, message: S, span: Span<'text, Nl>) -> Self 
        where S: Into<String>,
    {
        self.span = Some((span, message.into()));
        self
    }

    pub fn description(&self) -> &'static str {
        self.description
    }
}

impl<'text, Nl> ParseError<'text, Nl> 
    where Nl: NewLine,
{
    pub fn into_owned(self) -> ParseErrorOwned 
    {
        ParseErrorOwned {
            display: format!("{}", self),
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
            let source_name = "[SOURCE TEXT]".to_string();
            write!(f, "{}", 
                SourceSpan::new(span, &self.description)
                    .with_source_name(&source_name)
                    .with_highlight(Highlight::new(span, msg)))
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
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ParseErrorOwned
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseErrorOwned {
    display: String,
}


impl std::fmt::Display for ParseErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}
