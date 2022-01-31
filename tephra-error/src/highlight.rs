////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]



// Internal library imports.
use crate::MessageType;
use crate::RiserState;


// External library imports.
use colored::Color;
use colored::Colorize as _;
use tephra_span::ColumnMetrics;
use tephra_span::Span;
use tephra_span::SplitLines;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;

// Standard library imports.
use std::borrow::Cow;
use std::borrow::Borrow as _;
use std::fmt::Display;

////////////////////////////////////////////////////////////////////////////////
// Highlight
////////////////////////////////////////////////////////////////////////////////
/// A highlighted subsection of a `SourceSpan`.
#[derive(Debug)]
pub struct Highlight<'text, 'msg> {
    /// The span to highlight.
    span: Span<'text>,
    /// The message to display at the start of the span.
    start_message: Option<Cow<'msg, str>>,
    /// The message to display at the end of the span.
    end_message: Option<Cow<'msg, str>>,
    /// The message type.
    message_type: MessageType,
    /// Whether to allow line omissions within the highlighted span.
    allow_omissions: bool,
}


impl<'text, 'msg> Highlight<'text, 'msg> {
    /// Constructs a new Highlight with the given span and message.
    pub fn new<M>(span: Span<'text>, message: M) -> Self
        where M: Into<Cow<'msg, str>>,
    {
        Highlight {
            span,
            start_message: None,
            end_message: Some(message.into()),
            message_type: MessageType::Info,
            allow_omissions: true,
        }
    }

    /// Returns the given SourceDisplay with the info MessageType.
    pub fn with_info_type(mut self) -> Self {
        self.message_type = MessageType::Info;
        self
    }

    /// Returns the given SourceDisplay with the error MessageType.
    pub fn with_error_type(mut self) -> Self {
        self.message_type = MessageType::Error;
        self
    }

    /// Returns the given SourceDisplay with the warning MessageType.
    pub fn with_warning_type(mut self) -> Self {
        self.message_type = MessageType::Warning;
        self
    }
    
    /// Returns the given SourceDisplay with the note MessageType.
    pub fn with_note_type(mut self) -> Self {
        self.message_type = MessageType::Note;
        self
    }

    /// Returns the given SourceDisplay with the hel MessageType.
    pub fn with_help_type(mut self) -> Self {
        self.message_type = MessageType::Help;
        self
    }

    /// Returns the given SourceDisplay with the given MessageType.
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Returns true if the highlight extends across multiple lines.
    pub fn is_multiline(&self) -> bool {
        self.span.start().page.line != self.span.end().page.line
    }

    /// Returns true if the highlight has a message for the given line.
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
    pub(in crate) fn write_riser_for_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        current_line: usize,
        riser_state: &mut RiserState,
        is_active_riser: bool,
        color_enabled: bool)
        -> std::fmt::Result
    {
        let _span = span!(Level::TRACE, "write_riser_for_line").entered();

        // If the span is not over multiple lines, there is no riser portion.
        if *riser_state == RiserState::Unused { return Ok(()); }

        match *riser_state {
            RiserState::Unused  => Ok(()),

            RiserState::Ended   => write!(f, " "),

            RiserState::Waiting => if !is_active_riser 
                && self.span.start().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Started;
                if color_enabled {
                    write!(f, "{}", "/".color(self.message_type.color()))
                } else {
                    write!(f, "/")
                }

            } else if self.has_message_for_line(current_line) {
                *riser_state = RiserState::Started;
                write!(f, " ")

            } else {
                write!(f, " ")
            },

            RiserState::Started => if !is_active_riser 
                && self.span.end().page.column == 0
                && !self.has_message_for_line(current_line)
            {
                *riser_state = RiserState::Ended;
                if color_enabled {
                    write!(f, "{}", "\\".color(self.message_type.color()))
                } else {
                    write!(f, "\\")
                }

            } else if self.has_message_for_line(current_line) {
                if is_active_riser { *riser_state = RiserState::Ended; }
                write!(f, "|")

            } else {
                write!(f, "|")
            },
        }

    }

    /// Writes the message text for the given line number.
    pub(in crate) fn write_message_for_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        line: usize,
        write_extra_riser_spacer: bool,
        color_enabled: bool)
        -> std::fmt::Result
    {
        let _span = span!(Level::TRACE,
                "write_message_for_line",
                color_enabled)
            .entered();

        if self.span.start().page.line == line
            && self.span.end().page.line == line
        {
            if write_extra_riser_spacer { write!(f, " ")?; }
            for _ in 0..self.span.start().page.column {
                write!(f, " ")?;
            }
            if self.span.is_empty() {
                if color_enabled {
                    write!(f, "{}", "\\".color(self.message_type.color()))?;
                } else {
                    write!(f, "\\")?;
                }
            } else {
                let mut underline_count = std::cmp::max(
                    self.span.end().page.column
                        .checked_sub(self.span.start().page.column)
                        .unwrap_or(0),
                    1);
                for _ in 0..underline_count {
                    if color_enabled {
                        write!(f, "{}", self.message_type
                            .underline()
                            .color(self.message_type.color()))?;
                    } else {
                        write!(f, "{}", self.message_type.underline())?;
                    }
                }
            }
            match (&self.start_message, &self.end_message) {
                (Some(msg), None)      | 
                (None,      Some(msg)) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?
                } else {
                    writeln!(f, " {}", msg)?
                },
                (Some(fst), Some(snd)) => unimplemented!(),
                (None,      None)      => writeln!(f, "")?,
            }

        }  else if self.span.start().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(f, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(f, "_")?;
                }
            }
            if self.span.start().page.column > 0 {
                for _ in 0..(self.span.start().page.column - 1) {
                    if color_enabled {
                        write!(f, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(f, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(f, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(f, "^")?;
            }
            match &self.start_message {
                Some(msg) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?;
                } else {
                    write!(f, " {}", msg)?;
                },
                None      => writeln!(f, "")?,
            }
            
        } else if self.span.end().page.line == line {
            if write_extra_riser_spacer {
                if color_enabled {
                    write!(f, "{}", "_".color(self.message_type.color()))?;
                } else {
                    write!(f, "_")?;
                }
            }
            if self.span.end().page.column > 0 {
                for _ in 0..(self.span.end().page.column - 1) {
                    if color_enabled {
                        write!(f, "{}", "_".color(self.message_type.color()))?;
                    } else {
                        write!(f, "_")?;
                    }
                }
            }
            if color_enabled {
                write!(f, "{}", "^".color(self.message_type.color()))?;
            } else {
                write!(f, "^")?;
            }
            match &self.end_message {
                Some(msg) => if color_enabled {
                    writeln!(f, " {}", msg.color(self.message_type.color()))?;
                } else {
                    writeln!(f, " {}", msg)?;
                },
                
                None      => writeln!(f, "")?,
            }
        }
        Ok(())
    }
}
