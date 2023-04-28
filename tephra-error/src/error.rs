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
use crate::common::SourceError;

// External library imports.
use tephra_span::Span;
use tephra_span::SourceText;

// Standard library imports.
use std::error::Error;


////////////////////////////////////////////////////////////////////////////////
// Error traits.
////////////////////////////////////////////////////////////////////////////////

/// Provides a method to convert an error into an owned error.
pub trait ParseError<'text>: std::error::Error + 'text {
    /// Returns the span of the current parse when the failure occurred, if
    /// available.
    fn parse_span(&self) -> Option<Span> {
        None
    }

    /// Converts a ParseError<'text> into an owned error.
    fn into_owned(self: Box<Self>) -> Box<dyn std::error::Error + 'static>;
}

