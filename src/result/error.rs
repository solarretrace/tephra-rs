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
    span: Option<Span<'text, Nl>>,

}

impl<'text, Nl> ParseError<'text, Nl> {
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span<'text, Nl>) -> Self {
        self.span = Some(span);
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
        if let Some(span) = self.span {
            let source_name = "[SOURCE TEXT]".to_string();
            write!(f, "{}", 
                SourceSpan::new(span, &self.description)
                    .with_source_name(&source_name)
                    .with_highlight(Highlight::new(
                        span,
                        &"span start message")))
        } else {
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
