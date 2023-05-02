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
use tephra_span::SourceTextInner;
use tephra_span::SourceTextOwned;
use tephra_span::Span;


// Standard library imports.
use std::error::Error;
use std::fmt::Display;
use std::fmt::Debug;


////////////////////////////////////////////////////////////////////////////////
// SourceErrorInner
////////////////////////////////////////////////////////////////////////////////
pub type SourceError<'text> = SourceErrorInner<&'text str>;
pub type SourceErrorOwned = SourceErrorInner<Box<str>>;


/// A general-purpose error supporting formatted source text display.
#[derive(Debug)]
pub struct SourceErrorInner<T> where T: AsRef<str> {
    /// The source text.
    source_text: SourceTextInner<T>,
    /// The `CodeDisplay` used to format the error output.
    code_display: CodeDisplay,
    /// The underlying cause of the error.
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl<T> SourceErrorInner<T> where T: AsRef<str> {
    /// Constructs a new `SourceErrorInner` with the given `SourceText` and
    /// message.
    pub fn new<M>(source_text: SourceTextInner<T>, message: M)
        -> SourceErrorInner<T>
        where M: Into<String>,
    {
        SourceErrorInner {
            source_text,
            code_display: CodeDisplay::new(message)
                .with_error_type(),
            cause: None,
        }
    }

    /// Returns the given `SourceErrorInner` with the given error cause.
    pub fn with_cause(mut self, cause: Box<dyn Error + Send + Sync + 'static>)
        -> Self
    {
        self.cause = Some(cause);
        self
    }

    /// Returns the given `SourceErrorInner` with the given color enablement.
    pub fn with_color(mut self, color_enabled: bool) -> Self {
        self.code_display.color_enabled = color_enabled;
        self
    }

    /// Returns the given `SourceErrorInner` with the given note attachment.
    pub fn with_note<N>(mut self, note: N) -> Self
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
        self
    }

    /// Returns the given `SourceErrorInner` with the given note attachment.
    pub fn push_note<N>(&mut self, note: N)
        where N: Into<Note>
    {
        self.code_display.notes.push(note.into());
    }

    /// Returns the given `SourceErrorInner` with the given `SpanDisplay` attachment.
    pub fn with_span_display<S>(mut self, span_display: S) -> Self
        where S: Into<SpanDisplay>
    {
        self.code_display.push_span_display(span_display.into());
        self
    }

    /// Appends the given `SpanDisplay` to the `SourceErrorInner`.
    pub fn push_span_display<S>(&mut self, span_display: S)
        where S: Into<SpanDisplay>
    {
        self.code_display.span_displays.push(span_display.into());
    }

    /// Returns the `SourceErrorInner`'s message.
    pub fn message(&self) -> &str {
        self.code_display.message.as_str()
    }


    pub fn into_owned(self) -> SourceErrorOwned {
        SourceErrorInner {
            source_text: self.source_text.to_owned(),
            code_display: self.code_display,
            cause: self.cause,
        }
    }
}

impl<T> Display for SourceErrorInner<T> where T: AsRef<str> + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_display.write(f, self.source_text.borrow())
    }
}

impl<T> Error for SourceErrorInner<T> where T: AsRef<str> + Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}



impl<'text> ParseError<'text> for SourceError<'text> {
    fn into_source_error(self: Box<Self>, source_text: SourceText<'text>)
        -> SourceError<'text>
    {
        *self
    }

    fn into_owned(self: Box<Self> ) -> Box<dyn Error + Send + Sync + 'static> {
        Box::new(SourceErrorInner::into_owned(*self))
    }
}
