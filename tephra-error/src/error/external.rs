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
use crate::ParseError;
use crate::error::SourceErrorRef;

// External library imports.
use tephra_span::SourceTextRef;


// Standard library imports.
use std::num::ParseIntError;
use std::num::ParseFloatError;




////////////////////////////////////////////////////////////////////////////////
// std::num::ParseIntError
////////////////////////////////////////////////////////////////////////////////

impl ParseError for ParseIntError {
    fn into_source_error<'text>(
        self: Box<Self>,
        source_text: SourceTextRef<'text>)
        -> SourceErrorRef<'text>
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
    fn into_source_error<'text>(
        self: Box<Self>,
        source_text: SourceTextRef<'text>)
        -> SourceErrorRef<'text>
    {
        SourceErrorRef::new(source_text, format!("{self}"))
    }

    fn into_owned(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>
    {
        self
    }
}
