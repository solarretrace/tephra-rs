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
use crate::position::ColumnMetrics;
use crate::result::SourceDisplay;
use crate::result::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SectionType {
    Atomic     = 5,
    Bounded    = 4,
    Delimited  = 3,
    Unbounded  = 2,
    Validation = 1,
    Lexer      = 0,
}

impl Default for SectionType {
    fn default() -> Self {
        SectionType::Validation
    }
}

////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseError<'text> {
    description: &'static str,
    span: Option<(Span<'text>, String, ColumnMetrics)>,
    section_type: SectionType,
}

impl<'text> ParseError<'text> {
    pub fn new(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            section_type: SectionType::default(),
        }
    }

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

    pub fn with_section_type(mut self, section_type: SectionType) -> Self {
        self.section_type = section_type;
        self
    }

    pub fn unrecognized_token(span: Span<'text>, metrics: ColumnMetrics) -> Self {
        ParseError {
            description: "unrecognized token",
            span: Some((span, "symbol not recognized".to_owned(), metrics)),
            section_type: SectionType::Lexer,
        }
    }

    pub fn unexpected_token<T>(
        span: Span<'text>,
        expected: T,
        metrics: ColumnMetrics)
        -> Self 
        where T: std::fmt::Display
    {
        ParseError {
            description: "unexpected token",
            span: Some((span, format!("expected {}", expected), metrics)),
            section_type: SectionType::Lexer,
        }
    }

    pub fn unexpected_end_of_text(span: Span<'text>, metrics: ColumnMetrics)
        -> Self
    {
        ParseError {
            description: "unexpected end of text",
            span: Some((span, "text ends here".to_owned(), metrics)),
            section_type: SectionType::Lexer,
        }
    }

    pub fn expected_end_of_text(span: Span<'text>, metrics: ColumnMetrics) -> Self {
        ParseError {
            description: "expected end of text",
            span: Some((span, "text should end here".to_owned(), metrics)),
            section_type: SectionType::Lexer,
        }
    }

    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn section_type(&self) -> SectionType {
        self.section_type
    }
}

impl<'text> ParseError<'text> {
    pub fn into_owned(self) -> ParseErrorOwned 
    {
        ParseErrorOwned {
            description: self.description,
            span: self.span.map(|(a, b, c)| (a.into(), b, c)),
            section_type: self.section_type,
        }
    }
}

impl<'text> Default for ParseError<'text> {
    fn default() -> Self {
        ParseError {
            description: "parse error",
            span: None,
            section_type: SectionType::default(),
        }
    }
}

impl<'text> std::fmt::Display for ParseError<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((span, msg, metrics)) = &self.span {
            let span = *span;
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

impl<'text> From<&'static str> for ParseError<'text> {
    fn from(description: &'static str) -> Self {
        ParseError {
            description,
            span: None,
            section_type: SectionType::default(),
        }
    }
}

impl<'text> std::error::Error for ParseError<'text> {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}


////////////////////////////////////////////////////////////////////////////////
// ParseErrorOwned
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ParseErrorOwned {
    description: &'static str,
    span: Option<(SpanOwned, String, ColumnMetrics)>,
    section_type: SectionType,
}

impl ParseErrorOwned {
    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn section_type(&self) -> SectionType {
        self.section_type
    }
}

impl<'text> From<ParseError<'text>> for ParseErrorOwned {
    fn from(parse_error: ParseError<'text>) -> Self {
        ParseErrorOwned {
            description: parse_error.description,
            span: parse_error.span.map(|(sp, msg, cm)| (sp.into(), msg, cm)),
            section_type: parse_error.section_type,
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
        None
    }
}
