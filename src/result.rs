////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse results.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::span::Span;
use crate::span::OwnedSpan;

// Standard library imports.
use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'text, K, V>
    = Result<Success<'text, K, V>, Failure<'text, K>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for `ParseResult`s.
pub trait ParseResultExt {
}

impl<'text, K, V> ParseResultExt for ParseResult<'text, K, V> {
}

////////////////////////////////////////////////////////////////////////////////
// Success
////////////////////////////////////////////////////////////////////////////////
/// The result of a successful parse.
#[derive(Debug, Clone)]
pub struct Success<'text, K, V> {
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, K>,
    /// The span of the parse result.
    pub span: Span<'text>,
    /// The parsed value.
    pub value: V,
}


////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a failed parse with borrowed data.
pub struct Failure<'text, K> {
    /// The lexer state for continuing after the parse.
    pub lexer: Lexer<'text, K>,
    /// The span of the failed parse.
    pub span: Span<'text>,
    /// The failure reason.
    pub reason: Reason,
    /// The source of the failure.
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<'text, K> std::fmt::Debug for Failure<'text, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'text, K> std::fmt::Display for Failure<'text, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl<'text, K> std::error::Error for Failure<'text, K> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
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
            IncompleteParse { .. }     => true,
            UnexpectedToken            => true,
            UnexpectedEndOfText        => false,
            LexerError                 => true,
        }
    }
}
