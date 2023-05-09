////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! General-purpose errors supporting formatted source text display.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::error::SourceErrorRef;
use crate::ParseError;

// External library imports.
use tephra_span::SourceTextRef;

// Standard library imports.
use std::num::ParseFloatError;
use std::num::ParseIntError;


////////////////////////////////////////////////////////////////////////////////
// std::num::ParseIntError
////////////////////////////////////////////////////////////////////////////////

impl ParseError for ParseIntError {
    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceErrorRef::new(source_text, format!("{self}"))
    }

    fn into_owned(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>
    {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// std::num::ParseFloatError
////////////////////////////////////////////////////////////////////////////////

impl ParseError for ParseFloatError {
    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        SourceErrorRef::new(source_text, format!("{self}"))
    }

    fn into_owned(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>
    {
        self
    }
}
