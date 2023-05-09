////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::MessageType;
use crate::RiserState;

// External library imports.
use colored::Colorize as _;
use tephra_span::Span;

// Standard library imports.
use std::fmt::Write;


////////////////////////////////////////////////////////////////////////////////
// Highlight
////////////////////////////////////////////////////////////////////////////////
/// A highlighted subsection of a `SpanDisplay`.
#[derive(Debug, Clone)]
pub struct Highlight {
    /// The span to highlight.
    span: Span,
    /// The message to display at the start of the span.
    start_message: Option<String>,
    /// The message to display at the end of the span.
    end_message: Option<String>,
    /// The message type.
    message_type: MessageType,
    // TODO: Whether to allow line omissions within the highlighted span.
    _allow_omissions: bool,
}


impl Highlight {
    /// Constructs a new Highlight with the given span and message.
    pub fn new<M>(span: Span, message: M) -> Self
        where M: Into<String>,
    {
        Self {
            span,
            start_message: None,
            end_message: Some(message.into()),
            message_type: MessageType::Info,
            _allow_omissions: true,
        }
    }

    /// Returns the given `CodeDisplay` with the info `MessageType`.
    #[must_use]
    pub fn with_info_type(mut self) -> Self {
        self.message_type = MessageType::Info;
        self
    }

    /// Returns the given `CodeDisplay` with the error `MessageType`.
    #[must_use]
    pub fn with_error_type(mut self) -> Self {
        self.message_type = MessageType::Error;
        self
    }

    /// Returns the given `CodeDisplay` with the warning `MessageType`.
    #[must_use]
    pub fn with_warning_type(mut self) -> Self {
        self.message_type = MessageType::Warning;
        self
    }
    
    /// Returns the given `CodeDisplay` with the note `MessageType`.
    #[must_use]
    pub fn with_note_type(mut self) -> Self {
        self.message_type = MessageType::Note;
        self
    }

    /// Returns the given `CodeDisplay` with the hel `MessageType`.
    #[must_use]
    pub fn with_help_type(mut self) -> Self {
        self.message_type = MessageType::Help;
        self
    }

    /// Returns the given `CodeDisplay` with the given `MessageType`.
    #[must_use]
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Returns true if the highlight extends across multiple lines.
    #[must_use]
    pub fn is_multiline(&self) -> bool {
        self.span.start().page.line != self.span.end().page.line
    }

    /// Returns true if the highlight has a message for the given line.
    #[must_use]
    pub fn has_message_for_line(&self, line: usize) -> bool {
        (self.span.start().page.line == line
            && (self.start_message.is_some() 
                || self.span.start().page.column != 0))
        ||
        (self.span.end().page.line == line
            && (self.end_message.is_some() 
                || self.span.end().page.column != 0))
    }

    /// Writes the riser symbol for the given line number.
    pub(in crate) fn write_riser_for_line<W>(
        &self,
        out: &mut W,
        current_line: usize,
        riser_state: &mut RiserState,
        is_active_riser: bool,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        // If the span is not over multiple lines, there is no riser portion.
        if *riser_state == RiserState::Unused { return Ok(()); }

        match *riser_state {
            RiserState::Unused  => Ok(()),

            RiserState::Ended   => write!(out, " "),

            RiserState::Waiting => if !is_active_riser 
                && self.span.start().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Started;
                if color_enabled {
                    write!(out, "{}", "/".color(self.message_type.color()))
                } else {
                    write!(out, "/")
                }

            } else if self.has_message_for_line(current_line) {
                *riser_state = RiserState::Started;
                write!(out, " ")

            } else {
                write!(out, " ")
            },

            RiserState::Started => if !is_active_riser 
                && self.span.end().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Ended;
                if color_enabled {
                    write!(out, "{}", "\\".color(self.message_type.color()))
                } else {
                    write!(out, "\\")
                }

            } else if is_active_riser 
                && self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Ended;
                write!(out, "|")
            } else {
                write!(out, "|")
            },
        }
    }

    /// Writes the message text for the given line number.
    pub(in crate) fn write_message_for_line<W>(
        &self,
        out: &mut W,
        line: usize,
        write_extra_riser_spacer: bool,
        color_enabled: bool)
        -> std::fmt::Result
        where W: Write
    {
        if self.span.start().page.line == line
            && self.span.end().page.line == line
        {
            if write_extra_riser_spacer { write!(out, " ")?; }
            for _ in 0..self.span.start().page.column {
                write!(out, " ")?;
            }
            if self.span.is_empty() {
                if color_enabled {
                    write!(out, "{}", "\\".color(self.message_type.color()))?;
                } else {
                    write!(out, "\\")?;
                }
            } else {
                let underline_count = std::cmp::max(
                    self.span.end().page.column.saturating_sub(self.span.start().page.column),
                    1);
                for _ in 0..underline_count {
                    if color_enabled {
                        write!(out, "{}", self.message_type
                            .underline()
                            .color(self.message_type.color()))?;
                    } else {
                        write!(out, "{}", self.message_type
                            .underline())?;
                    }
                }
            }
            match (&self.start_message, &self.end_message) {
                (Some(msg), None)      | 
                (None,      Some(msg)) => if color_enabled {
                    writeln!(out, " {}", msg.color(self.message_type.color()))?
                } else {
                    writeln!(out, " {msg}")?
                },
                (Some(_fst), Some(_snd)) => todo!(),
                (None,      None)      => writeln!(out)?,
            }

        }  else if self.span.start().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(out, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(out, "_")?;
                }
            }
            if self.span.start().page.column > 0 {
                for _ in 0..(self.span.start().page.column - 1) {
                    if color_enabled {
                        write!(out, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(out, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(out, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(out, "^")?;
            }
            match &self.start_message {
                Some(msg) => if color_enabled {
                    writeln!(out, " {}", msg.color(self.message_type.color()))?;
                } else {
                    write!(out, " {msg}")?;
                },
                None      => writeln!(out)?,
            }
            
        } else if self.span.end().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(out, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(out, "_")?;
                }
            }
            if self.span.end().page.column > 0 {
                for _ in 0..(self.span.end().page.column - 1) {
                    if color_enabled {
                        write!(out, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(out, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(out, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(out, "^")?;
            }
            match &self.end_message {
                Some(msg) => if color_enabled {
                    writeln!(out, " {}", msg.color(self.message_type.color()))?;
                } else {
                    writeln!(out, " {msg}")?;
                },
                
                None      => writeln!(out)?,
            }
        }
        Ok(())
    }
}
