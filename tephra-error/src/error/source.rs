////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! General-purpose errors supporting formatted source text display.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Internal library imports.
use crate::ParseError;
use crate::Note;
use crate::SpanDisplay;
use crate::CodeDisplay;

// External library imports.
use tephra_span::SourceText;
use tephra_span::SourceTextOwned;
use tephra_span::Span;


// Standard library imports.
use std::error::Error;
use std::fmt::Display;
use std::fmt::Debug;



////////////////////////////////////////////////////////////////////////////////
// SourceError
////////////////////////////////////////////////////////////////////////////////
/// A general-purpose error supporting formatted source text display.
#[derive(Debug)]
pub struct SourceError<'text> {
    /// The source text.
    source_text: SourceText<'text>,
    /// The `CodeDisplay` used to format the error output.
    code_display: CodeDisplay,
    /// The underlying cause of the error.
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl<'text> SourceError<'text> {
    /// Attempt to convert an error into a `SourceError`. Works on common error
    /// types defined in the `tephra-error` crate.
    ///
    /// If the conversion fails, the error will be returned. The method is
    /// generic over the `Scanner`'s token type.
    pub fn try_from<T>(
        mut error: Box<dyn Error + Send + Sync + 'static>,
        source_text: SourceText<'text>)
        -> Result<Self, Box<dyn Error + Send + Sync + 'static>>
        where T: Debug + Display + Send + Sync + 'static,
    {
        error = match error
            .downcast::<crate::error::UnexpectedTokenError<T>>()
        {
            Ok(e)  => { return Ok(e.into_source_error(source_text)); }
            Err(e) => e,
        };

        error = match error
            .downcast::<crate::error::UnrecognizedTokenError>()
        {
            Ok(e)  => { return Ok(e.into_source_error(source_text)); }
            Err(e) => e,
        };

        error = match error
            .downcast::<crate::error::ParseBoundaryError>()
        {
            Ok(e)  => { return Ok(e.into_source_error(source_text)); }
            Err(e) => e,
        };

        error = match error
            .downcast::<crate::error::MatchBracketError>()
        {
            Ok(e)  => { return Ok(e.into_source_error(source_text)); }
            Err(e) => e,
        };

        error = match error
            .downcast::<crate::error::RepeatCountError>()
        {
            Ok(e)  => { return Ok(e.into_source_error(source_text)); }
            Err(e) => e,
        };

        Err(error)
    }

    /// Attempts to convert an error into a `SourceError` via
    /// `SourceError::try_from`, and returns it as an owned `dyn Error`.
    ///
    /// If the conversion fails, the error will be returned unchanged. The
    /// method is generic over the `Scanner`'s token type.
    pub fn convert<T>(
        error: Box<dyn Error + Send + Sync + 'static>,
        source_text: SourceText<'text>)
        -> Box<dyn Error + Send + Sync + 'static>
        where T: Debug + Display + Send + Sync + 'static,
    {
        match Self::try_from::<T>(error, source_text) {
            Ok(e) => Box::new(e).into_owned(),
            Err(e) => e,
        }
    }

    /// Constructs a new `SourceError` with the given `SourceText` and
    /// message.
    pub fn new<'a, M>(source_text: SourceText<'a>, message: M)
        -> SourceError<'a>
        where M: Into<String>,
    {
        SourceError {
            source_text,
            code_display: CodeDisplay::new(message)
                .with_error_type(),
            cause: None,
        }
    }

    /// Returns the given `SourceError` with the given error cause.
    pub fn with_cause(mut self, cause: Box<dyn Error + Send + Sync + 'static>)
        -> Self
    {
        self.cause = Some(cause);
        self
    }

    /// Returns the given `SourceError` with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.code_display.color_enabled = color_enabled;
        self
    }

    /// Returns the given `SourceError` with the given note attachment.
    pub fn with_note<N>(mut self, note: N) -> Self
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
        self
    }

    /// Returns the given `SourceError` with the given note attachment.
    pub fn push_note<N>(&mut self, note: N)
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
    }

    /// Returns the given `SourceError` with the given `SpanDisplay` attachment.
    pub fn with_span_display<S>(mut self, span_display: S) -> Self
        where S: Into<SpanDisplay>
    {
        self.code_display.push_span_display(span_display.into());
        self
    }

    /// Appends the given `SpanDisplay` to the `SourceError`.
    pub fn push_span_display<S>(&mut self, span_display: S)
        where S: Into<SpanDisplay>
    {
        self.code_display.span_displays.push(span_display.into());
    }

    /// Returns the `SourceError`'s message.
    pub fn message(&self) -> &str {
        self.code_display.message.as_str()
    }
}

impl<'text> Display for SourceError<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display.write(f, self.source_text)
    }
}

impl<'text> Error for SourceError<'text> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}

impl<'text> ParseError<'text> for SourceError<'text> {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        Box::new(SourceErrorOwned {
            source_text: self.source_text.to_owned(),
            code_display: self.code_display,
            cause: self.cause,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// SourceErrorOwned
////////////////////////////////////////////////////////////////////////////////
/// A general-purpose error supporting formatted source text display. An
/// owning variant of `SourceError`.
#[derive(Debug)]
pub struct SourceErrorOwned {
    source_text: SourceTextOwned,
    code_display: CodeDisplay,
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}


impl SourceErrorOwned {
    pub fn new<M>(source_text: SourceTextOwned, message: M)
        -> SourceErrorOwned
        where M: Into<String>,
    {
        SourceErrorOwned {
            source_text,
            code_display: CodeDisplay::new(message)
                .with_error_type(),
            cause: None,
        }
    }

    /// Returns the given `SourceErrorOwned` with the given error cause.
    pub fn with_cause(mut self, cause: Box<dyn Error + Send + Sync + 'static>)
        -> Self
    {
        self.cause = Some(cause);
        self
    }

    /// Returns the given `SourceErrorOwned` with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.code_display.color_enabled = color_enabled;
        self
    }

    /// Returns the given `SourceErrorOwned` with the given note attachment.
    pub fn with_note<N>(mut self, note: N) -> Self
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
        self
    }

    /// Returns the given `SourceErrorOwned` with the given note attachment.
    pub fn push_note<N>(&mut self, note: N)
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
    }
    
    /// Returns the given `SourceErrorOwned` with the given `SpanDisplay`
    /// attachment.
    pub fn with_span_display<S>(mut self, span_display: S) -> Self
        where S: Into<SpanDisplay>
    {
        self.code_display.push_span_display(span_display.into());
        self
    }

    /// Appends the given `SpanDisplay` to the `SourceErrorOwned`.
    pub fn push_span_display<S>(&mut self, span_display: S)
        where S: Into<SpanDisplay>
    {
        self.code_display.span_displays.push(span_display.into());
    }

    /// Returns the `SourceErrorOwned`'s message.
    pub fn message(&self) -> &str {
        self.code_display.message.as_str()
    }
}

impl Display for SourceErrorOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display.write(f, self.source_text.as_borrowed())
    }
}

impl Error for SourceErrorOwned {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}

impl<'text> ParseError<'text> for SourceErrorOwned {
    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}

