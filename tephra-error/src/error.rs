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
use crate::MessageType;

// External libary imports.
use tephra_span::ColumnMetrics;
use tephra_span::SourceText;
use tephra_span::SourceTextOwned;
use tephra_span::Span;
use tephra_tracing::event;
use tephra_tracing::Level;

use std::rc::Rc;



////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseError<'text> {
    /// The source text from which the error is derived.
    source_text: SourceText<'text>,
    /// The `CodeDisplay` for rendering the associated spans, highlights, and
    /// notes.
    code_display: CodeDisplay,
    /// The captured source of the parse error.
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text> ParseError<'text> {
    /// Constructs a new `ParseError` with the given description.
    pub fn new(source_text: SourceText<'text>, description: &'static str)
        -> Self
    {
        ParseError {
            source_text,
            code_display: CodeDisplay::new(description)
                .with_error_type(),
            source: None,
        }
    }

    /// Constructs an `unrecognized token` lexer error.
    pub fn unrecognized_token(source_text: SourceText<'text>, span: Span)
        -> Self
    {
        let e = ParseError {
            source_text,
            code_display: CodeDisplay::new("unrecognized token")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    span,
                    "symbol not recognized")),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `unexpected token` lexer error.
    pub fn unexpected_token<T>(
        source_text: SourceText<'text>, 
        span: Span,
        expected: T)
        -> Self 
        where T: std::fmt::Display
    {
        let e = ParseError {
            source_text,
            code_display: CodeDisplay::new("unexpected token")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    span,
                    format!("expected {expected}"))),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `unexpected end-of-text` lexer error.
    pub fn unexpected_end_of_text(source_text: SourceText<'text>, span: Span)
        -> Self
    {
        let e = ParseError {
            source_text,
            code_display: CodeDisplay::new("unexpected end of text")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    span,
                    "text ends here")),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `expected end-of-text` lexer error.
    pub fn expected_end_of_text(source_text: SourceText<'text>, span: Span)
        -> Self
    {
        let e = ParseError {
            source_text,
            code_display: CodeDisplay::new("expected end of text")
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    span,
                    "text should end here")),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Constructs an `unmatched_delimitter` lexer error.
    pub fn unmatched_delimitter<S>(
        source_text: SourceText<'text>,
        message: S,
        span: Span)
        -> Self
        where S: Into<String>
    {
        let e = ParseError {
            source_text,
            code_display: CodeDisplay::new(message)
                .with_error_type()
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    span,
                    "delimitter here is unmatched")),
            source: None,
        };
        event!(Level::TRACE, "{e:?}");
        e
    }

    /// Adds the given span to the `ParseError` and returns it.
    pub fn with_code_display(mut self, code_display: CodeDisplay)
        -> Self
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
    pub fn with_span_display<S>(mut self, span_display: S) -> Self
        where S: Into<SpanDisplay>
    {
        self.code_display.push_span_display(span_display);
        self
    }

    /// Returns the given ParseError with a note attachment.
    pub fn with_note<N>(mut self, note: N) -> Self
        where N: Into<Note>
    {
        self.code_display.push_note(note);
        self
    }

    /// Applies the given `ParseError` as a contextual wrapper around the
    /// contained error.
    pub fn with_error_context(mut self, error: Self) -> Self {
        self.push_error_context(error);
        self
    }

    /// Applies the given `ParseError` as a contextual wrapper around the
    /// contained error.
    pub fn push_error_context(&mut self, error: Self) {
        let source = std::mem::replace(self, error);
        self.source = Some(Box::new(source.into_owned()));
    }


    pub fn description(&self) -> &str {
        self.code_display.message()
    }
}

impl<'text> ParseError<'text> {
    pub fn into_owned(self) -> ParseErrorOwned {
        ParseErrorOwned::from(self)
    }
}

impl<'text> Default for ParseError<'text> {
    fn default() -> Self {
        ParseError {
            source_text: SourceText::empty(),
            code_display: CodeDisplay::new("parse error"),
            source: None,
        }
    }
}

impl<'text> std::fmt::Display for ParseError<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display
            .write_with_color_enablement(f, self.source_text, true)?;
        if let Some(source) = &self.source {
            write!(f, "... caused by {}", source)?;
        }
        Ok(())
    }
}

impl<'text> From<&'static str> for ParseError<'text> {
    fn from(description: &'static str) -> Self {
        ParseError {
            source_text: SourceText::empty(),
            code_display: CodeDisplay::new(description)
                .with_error_type(),
            source: None,
        }
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
    /// The source text from which the error is derived.
    source_text: SourceTextOwned,
    code_display: CodeDisplay,
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
            // TODO: Capture narrower span of text.
            source_text: parse_error.source_text.to_owned(),
            code_display: parse_error.code_display,
            source: parse_error.source,
        }
    }
}

impl std::fmt::Display for ParseErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display
            .write_with_color_enablement(
                f,
                self.source_text.as_borrowed(),
                true)?;
        if let Some(source) = &self.source {
            write!(f, "... caused by {}", source)?;
        }
        Ok(())
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

