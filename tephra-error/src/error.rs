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
use crate::CodeDisplay;
use crate::Note;
use crate::SpanDisplay;

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
    // TODO: Generalize this.
    /// The `CodeDisplay` for rendering the associated spans, highlights, and
    /// notes.
    code_display: CodeDisplay<'text>,
    /// The captured source of the parse error.
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text> ParseError<'text> {
    /// Constructs a new `ParseError` with the given description.
    pub fn new(description: &'static str) -> Self {
        ParseError {
            code_display: CodeDisplay::new(description)
                .with_error_type(),
            source: None,
        }
    }

    /// Constructs an `unrecognized token` lexer error.
    pub fn unrecognized_token(span: Span<'text>, metrics: ColumnMetrics) -> Self {
        let e = ParseError {
            code_display: CodeDisplay::new("unrecognized token")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    span,
                    "symbol not recognized",
                    metrics)),
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
            code_display: CodeDisplay::new("unexpected token")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    span,
                    format!("expected {expected}"),
                    metrics)),
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
            code_display: CodeDisplay::new("unexpected end of text")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    span,
                    "text ends here",
                    metrics)),
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
            code_display: CodeDisplay::new("expected end of text")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    span,
                    "text should end here",
                    metrics)),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Adds the given span to the `ParseError` and returns it.
    pub fn with_code_display<S>(
        mut self,
        code_display: CodeDisplay<'text>)
        -> Self 
        where S: Into<String>,
    {
        self.code_display = code_display;
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


    /// Returns the given ParseError with a SpanDisplay attachment.
    pub fn with_span_display<S>(mut self, span_display: S)
        -> Self
        where S: Into<SpanDisplay<'text>>
    {
        self.code_display.push_span_display(span_display);
        self
    }

    /// Returns the given ParseError with a note attachment.
    pub fn with_note<N>(mut self, note: N)
        -> Self
        where N: Into<Note>
    {
        self.code_display.push_note(note);
        self
    }

    pub fn with_context(mut self, description: &'static str) -> Self {
        self.push_context(description);
        self
    }

    pub fn push_context(&mut self, description: &'static str) {
        let source = std::mem::replace(self, ParseError {
            code_display: CodeDisplay::new(description),
            source: None,
        });
        self.source = Some(Box::new(source.into_owned()));
    }


    pub fn push_context_error(&mut self, context: Self) {
        let source = std::mem::replace(self, context);
        self.source = Some(Box::new(source.into_owned()));
    }


    pub fn description(&self) -> &str {
        self.code_display.message()
    }
}

impl<'text> ParseError<'text> {
    pub fn into_owned(self) -> ParseErrorOwned 
    {
        ParseErrorOwned::from(self)
    }
}

impl<'text> Default for ParseError<'text> {
    fn default() -> Self {
        ParseError {
            code_display: CodeDisplay::new("parse error"),
            source: None,
        }
    }
}

impl<'text> std::fmt::Display for ParseError<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code_display)
    }
}

impl<'text> From<&'static str> for ParseError<'text> {
    fn from(description: &'static str) -> Self {
        ParseError::new(description)
    }
}

impl<'text> std::error::Error for ParseError<'text> {
    fn description(&self) -> &str {
        self.description()
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
    description: Box<str>,
    code_display: Box<str>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ParseErrorOwned {
    pub fn description(&self) -> &str {
        self.description.as_ref()
    }
}

impl<'text> From<ParseError<'text>> for ParseErrorOwned {
    fn from(parse_error: ParseError<'text>) -> Self {
        ParseErrorOwned {
            description: parse_error.description().into(),
            code_display: format!("{}", parse_error.code_display).into(),
            source: parse_error.source,
        }
    }
}

impl std::fmt::Display for ParseErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code_display)
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
