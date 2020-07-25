////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Result for a failed parse.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::span::Span;
use crate::span::OwnedSpan;
use crate::result::display::*;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with borrowed data.
pub struct Failure<'text, S> where S: Scanner {
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, S>,
    /// The span of the failed parse.
    pub span: Span<'text>,
    /// The failure reason.
    pub reason: Reason,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}


impl<'text, S> std::fmt::Debug for Failure<'text, S> where S: Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'text, S> std::fmt::Display for Failure<'text, S> where S: Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_error_line(f, &self.reason)?;
        write_source_info_line(f, "[text]", self.span)?;
        write_gutter(f, 3, None)?;
        writeln!(f, "")?;
        write_gutter(f, 3, Some(1))?;
        writeln!(f, "")?;
        write_gutter(f, 3, None)?;
        writeln!(f, "")

    }
}

impl<'text, S> std::error::Error for Failure<'text, S> where S: Scanner {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

#[cfg(test)]
impl<'text, S> PartialEq for Failure<'text, S> where S: Scanner {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}


////////////////////////////////////////////////////////////////////////////////
// FailureOwned
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with owned data.
///
/// Similar to [`Failure`], except this  version owns all of its data, and can
/// thus  be used as an [`Error`] [`source`].
///
/// [`Failure`]: struct.Failure.html
/// [`Error`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
/// [`source`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html#method.source
#[derive(Debug)]
pub struct FailureOwned {
    /// The span of the failed parse.
    pub span: OwnedSpan,
    /// The failure reason.
    pub reason: Reason,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text, S> From<Failure<'text, S>> for FailureOwned where S: Scanner {
    fn from(other: Failure<'text, S>) -> Self {
        FailureOwned {
            span: other.span.into_owned(),
            reason: other.reason,
            source: other.source,
        }
    }
}

impl std::fmt::Display for FailureOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for FailureOwned {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}



////////////////////////////////////////////////////////////////////////////////
// Reason
////////////////////////////////////////////////////////////////////////////////
/// The reason for a parse failure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    /// A parse was not completed due to another error.
    IncompleteParse {
        /// A description of the parse which was left incomplete.
        context: Cow<'static, str>,
    },
    /// An unexpected token was encountered.
    UnexpectedToken,
    /// The end of the text was unexpectedly encountered.
    UnexpectedEndOfText,
    /// A lexer error occurred.
    LexerError,
}

impl Reason {
    /// Constructs a Reason::IncompleteParse using the given string.
    pub fn incomplete<S>(msg: S) -> Self
        where S: Into<Cow<'static, str>>,
    {
        Reason::IncompleteParse {
            context: msg.into(),
        }
    }

    /// Returns true if the failure is of a recoverable kind.
    pub fn is_recoverable(&self) -> bool {
        use Reason::*;
        match self {
            IncompleteParse { .. } => true,
            UnexpectedToken        => true,
            UnexpectedEndOfText    => false,
            LexerError             => true,
        }
    }
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Reason::*;
        match self {
            IncompleteParse { .. } => write!(f, "Incomplete parse"),
            UnexpectedToken        => write!(f, "Unexpected token"),
            UnexpectedEndOfText    => write!(f, "Unexpected end of text"),
            LexerError             => write!(f, "Lexer error"),
        }
    }
}
