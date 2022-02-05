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
// MessageType
////////////////////////////////////////////////////////////////////////////////
/// A `CodeDisplay`, `SourceNote`, or `Highlight` message type. Used to
/// determine the color and format of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// An informational message.
    Info,
    /// An error message.
    Error,
    /// A warning message.
    Warning,
    /// A message providing additional info.
    Note,
    /// A message to help in correcting an error or warning.
    Help,
}

impl MessageType {
    /// Returns the color associated with the message type.
    pub fn color(&self) -> Color {
        use MessageType::*;
        match self {
            Info    => Color::BrightWhite,
            Error   => Color::BrightRed,
            Warning => Color::BrightYellow,
            Note    => Color::BrightBlue,
            Help    => Color::BrightGreen,
        }
    }

    /// Returns the underline associated with the message type.
    pub fn underline(&self) -> &'static str {
        use MessageType::*;
        match self {
            Info    => "-",
            Error   => "^",
            Warning => "^",
            Note    => "-",
            Help    => "~",
        }
    }

    pub(in crate) fn write_with_color_enablement(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        color_enabled: bool)
        -> std::fmt::Result
    {
        let _span = span!(Level::TRACE, "MessageType", color_enabled).entered();

        use MessageType::*;
        if color_enabled {
            let color = self.color();
            event!(Level::TRACE, "{self:?}");
            
            match self {
                Info    => write!(f, "{}", "info"),
                Error   => write!(f, "{}", "error".color(color).bold()),
                Warning => write!(f, "{}", "warning".color(color).bold()),
                Note    => write!(f, "{}", "note".color(color).bold()),
                Help    => write!(f, "{}", "help".color(color).bold()),
            }
        } else {
            match self {
                Info    => write!(f, "info"),
                Error   => write!(f, "error"),
                Warning => write!(f, "warning"),
                Note    => write!(f, "note"),
                Help    => write!(f, "help"),
            }
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_with_color_enablement(f, true)
    }
}
