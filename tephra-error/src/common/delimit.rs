////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common delimitter combinator errors.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Internal library imports.
use crate::IntoErrorOwned;
use crate::Note;
use crate::SpanDisplay;
use crate::CodeDisplay;
use crate::Highlight;
use crate::common::SourceError;

// External library imports.
use tephra_span::SourceText;
use tephra_span::Span;
use tephra_span::Pos;


// Standard library imports.
use std::error::Error;
use std::fmt::Display;
use std::fmt::Debug;
use std::fmt::Write;
use std::iter::IntoIterator;



////////////////////////////////////////////////////////////////////////////////
// IncompleteParseError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a successful parse does not consume as much text as
/// required.
#[derive(Debug, Clone)]
pub struct IncompleteParseError {
    /// The start position of the parse.
    pub start_pos: Pos,
    /// The actual end position of the parse.
    pub actual_end_pos: Pos,
    /// The expected end position of the parse.
    pub expected_end_pos: Pos,
}

impl IncompleteParseError {
    /// Returns the span of the parsed text.
    pub fn parsed_span(&self) -> Span {
        Span::new_enclosing(
            self.start_pos,
            self.actual_end_pos)
    }

    /// Returns the full span of the parsed and unexpected text.
    pub fn full_span(&self) -> Span {
        Span::new_enclosing(
            self.start_pos,
            self.expected_end_pos)
    }

    /// Returns the span of the unexpected text.
    pub fn unparsed_span(&self) -> Span {
        Span::new_enclosing(
            self.actual_end_pos,
            self.expected_end_pos)
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    pub fn into_source_error<'text>(self, source_text: SourceText<'text>)
        -> SourceError<'text>
    {
        SourceError::new(source_text, "incomplete parse")
            .with_span_display(SpanDisplay::new(
                    source_text,
                    self.full_span())
                .with_highlight(Highlight::new(
                    self.unparsed_span(),
                    "unexpected text")))
            .with_cause(Box::new(self))
    }
}

impl Display for IncompleteParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incomplete parse: unexpected text at {}",
            self.unparsed_span())
    }
}

impl Error for IncompleteParseError {}

impl IntoErrorOwned for IncompleteParseError {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + 'static> {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// BracketError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a successful parse does not consume as much text as
/// required.
#[derive(Debug, Clone)]
pub enum BracketError {
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

impl BracketError {
    /// Returns the full span of the error.
    fn full_span(&self) -> Span {
        use BracketError::*;
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
    pub fn into_source_error<'text>(self, source_text: SourceText<'text>)
        -> SourceError<'text>
    {
        use BracketError::*;
        let mut source_error = match self {
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
                        "... does not match the closing bracket here"))),
        };

        source_error.with_cause(Box::new(self))
    }
}

impl Display for BracketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BracketError::*;
        match self {
            NoneFound { expected_start } => write!(f,
                "bracket error: expected open bracket at {}",
                expected_start),

            Unclosed { found_start } => write!(f,
                "bracket error: unmatched open bracket at {}",
                found_start),
            
            Unopened { found_end } => write!(f,
                "bracket error: unmatched close bracket at {}",
                found_end),
            
            Mismatch { found_start, found_end } => write!(f,
                "bracket error: mismatched brackets at {} and {}",
                found_start,
                found_end),
        }
    }
}

impl Error for BracketError {}

impl IntoErrorOwned for BracketError {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + 'static> {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
// ItemCountError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a repeated parse fails to occur the required number
/// of times.
#[derive(Debug, Clone)]
pub struct ItemCountError {
    pub items_span: Span,
    pub found: usize,
    pub expected_min: usize,
    pub expected_max: Option<usize>,
}


impl ItemCountError {
    fn expected_description(&self) -> String {
        if self.found < self.expected_min {
            format!("expected {} item{}; found {}",
                self.expected_min,
                if self.expected_min == 1 { "" } else { "s" },
                self.found)
        } else {
            let max = self.expected_max.expect("get max item count");
            if max != self.expected_min {
                format!("expected {} to {} items; found {}",
                    self.expected_min,
                    max,
                    self.found)
            } else {
                format!("expected {} item{}; found {}",
                    max,
                    if max == 1 { "" } else { "s" },
                    self.found)
            }
        }
    }

    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    pub fn into_source_error<'text>(self, source_text: SourceText<'text>)
        -> SourceError<'text>
    {
        SourceError::new(source_text, "invalid item count")
            .with_span_display(SpanDisplay::new(
                    source_text,
                    self.items_span)
                .with_highlight(Highlight::new(
                    self.items_span,
                    self.expected_description())))
            .with_cause(Box::new(self))
    }
}

impl Display for ItemCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid item count: {}",
            self.expected_description())
    }
}

impl Error for ItemCountError {}

impl IntoErrorOwned for ItemCountError {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + 'static> {
        self
    }
}
