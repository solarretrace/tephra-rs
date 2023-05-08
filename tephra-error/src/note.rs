////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! SpanDisplay note attachments.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::MessageType;

// Standard library imports.
use std::fmt::Display;
use std::fmt::Write;


////////////////////////////////////////////////////////////////////////////////
// Note
////////////////////////////////////////////////////////////////////////////////
/// A note which can be attached to a `SpanDisplay` or `CodeDisplay`.
#[derive(Debug, Clone)]
pub struct Note {
    /// The message type for the note.
    pub(in crate) note_type: MessageType,
    /// The note to display.
    pub(in crate) note: String,
}

impl Note {
    pub(in crate) fn write_with_color_enablement<W>(
        &self,
        out: &mut W,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        self.note_type.write_with_color_enablement(out, color_enabled)?;
        write!(out, ": {}", self.note)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, true)
    }
}

