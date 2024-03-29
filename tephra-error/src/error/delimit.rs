////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common delimitter combinator errors.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::error::SourceError;
use crate::error::SourceErrorRef;
use crate::Highlight;
use crate::ParseError;
use crate::SpanDisplay;

// External library imports.
use tephra_span::Pos;
use tephra_span::SourceTextRef;
use tephra_span::Span;


// Standard library imports.
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;


////////////////////////////////////////////////////////////////////////////////
// ParseBoundaryError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a successful parse does not consume as much text as
/// required.
#[derive(Debug, Clone, Copy)]
pub struct ParseBoundaryError {
    /// The span of the parse up to the expected parse boundary.
    pub error_span: Span,
    /// The expected end position of the parse.
    pub expected_end_pos: Pos,
}

impl ParseBoundaryError {
    /// Returns the full span of the parsed and unexpected text.
    #[must_use]
    pub fn full_span(&self) -> Span {
        Span::enclosing(
            self.error_span.start(),
            self.expected_end_pos)
    }

    /// Returns the span of the unexpected text.
    #[must_use]
    pub fn unparsed_span(&self) -> Span {
        Span::enclosing(
            self.error_span.end(),
            self.expected_end_pos)
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    #[must_use]
    pub fn into_source_error(self, source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceError::new(source_text, "incomplete parse")
            .with_span_display(SpanDisplay::new(
                    source_text,
                    self.full_span())
                .with_highlight(Highlight::new(
                        self.unparsed_span(),
                        "unexpected text")
                    .with_error_type()))
            .with_cause(Box::new(self))
    }
}

impl Display for ParseBoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incomplete parse: unexpected text at {}",
            self.unparsed_span())
    }
}

impl Error for ParseBoundaryError {}

impl ParseError for ParseBoundaryError {
    fn error_span(&self) -> Option<Span> {
        Some(self.error_span)
    }

    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// MatchBracketError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a successful parse does not consume as much text as
/// required.
#[derive(Debug, Clone, Copy)]
pub enum MatchBracketError {
    NoneFound {
        expected_start: Span,
    },
    Unclosed {
        found_start: Span,
    },
    Unopened {
        found_end: Span,
    },
    Mismatch {
        found_start: Span,
        found_end: Span,
    },
}

impl MatchBracketError {
    /// Returns the full span of the error.
    fn full_span(&self) -> Span {
        use MatchBracketError::*;
        match self {
            NoneFound { expected_start } => *expected_start,
            Unclosed { found_start }     => *found_start,
            Unopened { found_end }       => *found_end,

            Mismatch { found_start, found_end }
                => found_start.enclose(*found_end),
        }
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    #[must_use]
    pub fn into_source_error(self, source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        use MatchBracketError::*;
        let source_error = match self {
            NoneFound { expected_start } => SourceError::new(
                    source_text,
                    "expected open bracket")
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    expected_start,
                    "bracket expected here")),

            Unclosed { found_start } => SourceError::new(
                    source_text,
                    "unmatched open bracket")
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    found_start,
                    "this bracket is not closed")),

            Unopened { found_end } => SourceError::new(
                    source_text,
                    "unmatched close bracket")
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    found_end,
                    "this bracket has no matching open")),

            Mismatch { found_start, found_end } => SourceError::new(
                    source_text,
                    "mismatched brackets")
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    found_start,
                    "the bracket here")
                    .with_highlight(Highlight::new(
                            found_end,
                            "... does not match the closing bracket here")
                        .with_error_type())),
        };

        source_error.with_cause(Box::new(self))
    }
}

impl Display for MatchBracketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MatchBracketError::*;
        match self {
            NoneFound { expected_start } => write!(f,
                "bracket error: expected open bracket at {expected_start}"),

            Unclosed { found_start } => write!(f,
                "bracket error: unmatched open bracket at {found_start}"),
            
            Unopened { found_end } => write!(f,
                "bracket error: unmatched close bracket at {found_end}"),
            
            Mismatch { found_start, found_end } => write!(f,
                "bracket error: mismatched brackets at {found_start} and {found_end}"),
        }
    }
}

impl Error for MatchBracketError {}

impl ParseError for MatchBracketError {
    fn error_span(&self) -> Option<Span> {
        Some(self.full_span())
    }

    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
// RepeatCountError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a repeated parse fails to occur the required number
/// of times.
#[derive(Debug, Clone, Copy)]
pub struct RepeatCountError {
    /// The span of the parse up to the start of the invalid count.
    pub error_span: Span,
    /// The number of parsed items found.
    pub found: usize,
    /// The minimum expected number of parsed items.
    pub expected_min: usize,
    /// The maximum expected number of parsed items.
    pub expected_max: Option<usize>,
}


impl RepeatCountError {
    fn expected_description(&self) -> String {
        if self.found < self.expected_min {
            format!("expected {} item{}; found {}",
                self.expected_min,
                if self.expected_min == 1 { "" } else { "s" },
                self.found)
        } else {
            let max = self.expected_max.expect("get max item count");
            if max == self.expected_min {
                format!("expected {} item{}; found {}",
                    max,
                    if max == 1 { "" } else { "s" },
                    self.found)
            } else {
                format!("expected {} to {} items; found {}",
                    self.expected_min,
                    max,
                    self.found)
            }
        }
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    #[must_use]
    pub fn into_source_error(self, source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceError::new(source_text, "invalid item count")
            .with_span_display(SpanDisplay::new(
                    source_text,
                    self.error_span)
                .with_highlight(Highlight::new(
                        self.error_span,
                        self.expected_description())
                    .with_error_type()))
            .with_cause(Box::new(self))
    }
}

impl Display for RepeatCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid item count: {}",
            self.expected_description())
    }
}

impl Error for RepeatCountError {}

impl ParseError for RepeatCountError {
    fn error_span(&self) -> Option<Span> {
        Some(self.error_span)
    }

    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}
