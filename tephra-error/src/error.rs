////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse error.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Internal library imports.
use crate::SourceDisplay;
use crate::SourceSpan;

// External libary imports.
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_span::Span;
use tephra_span::SpanOwned;
use tephra_span::ColumnMetrics;


////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseError<'text> {
    /// A description of the parse error.
    description: &'static str,
    // TODO: Generalize this.
    /// The span, message, and ColumnMetrics for the error highlight.
    span: Option<(Span<'text>, String, ColumnMetrics)>,
    /// The captured source of the parse error.
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text> ParseError<'text> {
    /// Constructs a new `ParseError` with the given description.
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            source: None,
        }
    }

    /// Constructs an `unrecognized token` lexer error.
    pub fn unrecognized_token(span: Span<'text>, metrics: ColumnMetrics) -> Self {
        let e = ParseError {
            description: "unrecognized token",
            span: Some((span, "symbol not recognized".to_owned(), metrics)),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `unexpected token` lexer error.
    pub fn unexpected_token<T>(
        span: Span<'text>,
        expected: T,
        metrics: ColumnMetrics)
        -> Self 
        where T: std::fmt::Display
    {
        let e = ParseError {
            description: "unexpected token",
            span: Some((span, format!("{expected}"), metrics)),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `unexpected end-of-text` lexer error.
    pub fn unexpected_end_of_text(span: Span<'text>, metrics: ColumnMetrics)
        -> Self
    {
        let e = ParseError {
            description: "unexpected end of text",
            span: Some((span, "text ends here".to_owned(), metrics)),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `expected end-of-text` lexer error.
    pub fn expected_end_of_text(span: Span<'text>, metrics: ColumnMetrics)
        -> Self
    {
        let e = ParseError {
            description: "expected end of text",
            span: Some((span, "text should end here".to_owned(), metrics)),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Adds the given span to the `ParseError` and returns it.
    pub fn with_span<S>(
        mut self,
        message: S,
        span: Span<'text>,
        metrics: ColumnMetrics)
        -> Self 
        where S: Into<String>,
    {
        self.span = Some((span, message.into(), metrics));
        self
    }

    /// Adds the given error source to the `ParseError` and returns it.
    pub fn with_source(
        mut self, 
        source: Box<dyn std::error::Error + Send + Sync + 'static>)
        -> Self
    {
        self.source = Some(source);
        self
    }

    pub fn with_context(mut self, description: &'static str) -> Self {
        self.push_context(description);
        self
    }

    pub fn push_context(&mut self, description: &'static str) {
        let source = std::mem::replace(self, ParseError {
            description,
            span: None,
            source: None,
        });
        self.source = Some(Box::new(source.into_owned()));
    }


    pub fn push_context_error(&mut self, context: Self) {
        let source = std::mem::replace(self, context);
        self.source = Some(Box::new(source.into_owned()));
    }


    pub fn description(&self) -> &'static str {
        self.description
    }
}

impl<'text> ParseError<'text> {
    pub fn into_owned(self) -> ParseErrorOwned 
    {
        ParseErrorOwned {
            description: self.description,
            span: self.span.map(|(a, b, c)| (a.into(), b, c)),
            source: self.source,
        }
    }
}

impl<'text> Default for ParseError<'text> {
    fn default() -> Self {
        ParseError {
            description: "parse error",
            span: None,
            source: None,
        }
    }
}

impl<'text> std::fmt::Display for ParseError<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((span, msg, metrics)) = &self.span {
            let span = *span;
            let msg = &*msg;
            let metrics = *metrics;
            write!(f, "{}", SourceDisplay::new(self.description)
                .with_error_type()
                .with_source_span(
                    SourceSpan::new_error_highlight(span, msg, metrics)))
        } else {
            write!(f, "{}", SourceDisplay::new(self.description)
                .with_error_type())
        }
    }
}

impl<'text> From<&'static str> for ParseError<'text> {
    fn from(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            source: None,
        }
    }
}

impl<'text> std::error::Error for ParseError<'text> {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}


////////////////////////////////////////////////////////////////////////////////
// ParseErrorOwned
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseErrorOwned {
    description: &'static str,
    span: Option<(SpanOwned, String, ColumnMetrics)>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ParseErrorOwned {
    pub fn description(&self) -> &'static str {
        self.description
    }
}

impl<'text> From<ParseError<'text>> for ParseErrorOwned {
    fn from(parse_error: ParseError<'text>) -> Self {
        ParseErrorOwned {
            description: parse_error.description,
            span: parse_error.span.map(|(sp, msg, cm)| (sp.into(), msg, cm)),
            source: parse_error.source,
        }
    }
}

impl std::fmt::Display for ParseErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((span, msg, metrics)) = &self.span {
            let span = span.into();
            let msg = &*msg;
            let metrics = *metrics;
            write!(f, "{}", 
                SourceDisplay::new(self.description)
                    .with_error_type()
                    .with_source_span(
                        SourceSpan::new_error_highlight(span, msg, metrics)))
        } else {
            // TODO: Clean up message.
            write!(f, "{} NO SPAN", self.description)
        }
    }
}

impl std::error::Error for ParseErrorOwned {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}
