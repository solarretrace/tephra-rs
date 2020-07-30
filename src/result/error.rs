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
use crate::span::SpanOwned;
use crate::span::NewLine;
use crate::result::SourceSpan;
use crate::result::Highlight;


#[derive(Debug)]
pub struct ParseError<'text, Nl> {
    description: &'static str,
    _span: Option<Span<'text, Nl>>,
}

impl<'text, Nl> ParseError<'text, Nl> {
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            _span: None,
        }
    }

    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn into_owned(self, span: Span<'text, Nl>) -> ParseErrorOwned {
        ParseErrorOwned {
            span: span.into_owned(),
        }
    }

    pub fn write_display(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        span: Span<'text, Nl>)
        -> std::fmt::Result
        where Nl: NewLine,
    {
        let source_name = "[SOURCE TEXT]".to_string();
        write!(f, "{}", 
            SourceSpan::new(span, &self.description)
                .with_source_name(&source_name)
                .with_highlight(Highlight::new(
                    span,
                    &"span start message")))
    }
}



#[derive(Debug)]
pub struct ParseErrorOwned {
    span: SpanOwned,
}

