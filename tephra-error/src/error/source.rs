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
use crate::CodeDisplay;
use crate::Note;
use crate::SpanDisplay;


// External library imports.
use tephra_span::SourceText;


// Standard library imports.
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;


////////////////////////////////////////////////////////////////////////////////
// SourceError
////////////////////////////////////////////////////////////////////////////////
pub type SourceErrorRef<'text> = SourceError<&'text str>;
pub type SourceErrorOwned = SourceError<Box<str>>;


/// A general-purpose error supporting formatted source text display.
#[derive(Debug)]
pub struct SourceError<T> where T: AsRef<str> {
    /// The source text.
    source_text: SourceText<T>,
    /// The `CodeDisplay` used to format the error output.
    code_display: CodeDisplay,
    /// The underlying cause of the error.
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl<T> SourceError<T> where T: AsRef<str> {
    /// Constructs a new `SourceError` with the given `SourceText` and
    /// message.
    #[must_use]
    pub fn new<M>(source_text: SourceText<T>, message: M)
        -> Self
        where M: Into<String>,
    {
        Self {
            source_text,
            code_display: CodeDisplay::new(message)
                .with_error_type(),
            cause: None,
        }
    }

    /// Returns the given `SourceError` with the given error cause.
    #[must_use]
    pub fn with_cause(mut self, cause: Box<dyn Error + Send + Sync + 'static>)
        -> Self
    {
        self.cause = Some(cause);
        self
    }

    /// Returns the given `SourceError` with the given color enablement.
    #[must_use]
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.code_display.color_enabled = color_enabled;
        self
    }

    /// Returns the given `SourceError` with the given note attachment.
    #[must_use]
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
    #[must_use]
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


    #[must_use]
    pub fn into_owned(self) -> SourceErrorOwned {
        SourceError {
            source_text: self.source_text.to_owned(),
            code_display: self.code_display,
            cause: self.cause,
        }
    }
}

impl<T> Display for SourceError<T> where T: AsRef<str> + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display.write(f, self.source_text.borrow())
    }
}

impl<T> Error for SourceError<T> where T: AsRef<str> + Debug + Display {
    #[allow(trivial_casts)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}

