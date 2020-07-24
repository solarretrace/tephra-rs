////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::span::Span;
use crate::span::Pos;

// Standard library imports.
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////
// Lexeme
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text associated with a lexed token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lexeme<'text, T> {
    /// The parsed token.
    token: T,
    /// The span of the parsed text.
    span: Span<'text>
}

impl<'text, T> Lexeme<'text, T> where T: PartialEq {
    /// Returns true if the token was parsed from whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.span.text().chars().all(char::is_whitespace)
    }

    /// Returns a reference to the lexed token.
    pub fn token(&self) -> &T {
        &self.token
    }

    /// Returns a reference to the lexed token's span.
    pub fn span(&self) -> &Span<'text> {
        &self.span
    }

    /// Consumes the lexeme and returns its span.
    pub fn into_span(self) -> Span<'text> {
        self.span
    }
}

impl<'text, T> PartialEq<T> for Lexeme<'text, T> where T: PartialEq {
    fn eq(&self, other: &T) -> bool {
        self.token == *other
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'text, K> {
    text: &'text str,
    pos: Pos,
    inner: K,
}

impl<'text, K> Lexer<'text, K> {
    /// Constructs a new Lexer for the given text and span newlines.
    pub fn new(inner: K, text: &'text str) -> Self {
        Lexer {
            text,
            pos: Pos::ZERO,
            inner,
        }
    }

    /// Returns the current lexer position.
    pub fn current_pos(&self) -> Pos {
        self.pos
    }

    /// Returns the empty span of the current lexer position.
    pub fn current_span(&self) -> Span<'text> {
        Span::new_from(self.pos, self.text)
    }

    /// Returns the span of all previously lexed text.
    pub fn lexed_span(&self) -> Span<'text> {
        let mut span = Span::new(self.text);
        span.extend_by(self.pos);
        span
    }
}

impl<'text, K> Iterator for Lexer<'text, K>
    where K: Tokenize,
{
    type Item = Result<Lexeme<'text, K::Token>, K::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.byte >= self.text.len() {
            return None;
        }

        match self.inner.parse_token(&self.text[self.pos.byte..]) {
            Ok((token, skip)) => {
                let mut span = Span::new_from(self.pos, self.text);
                span.extend_by(skip);
                self.pos = span.end_position();
                Some(Ok(Lexeme { token, span }))
            },

            Err((error, skip)) => {
                if skip.is_zero() { self.pos.byte = self.text.len() }

                let mut span = Span::new_from(self.pos, self.text);
                span.extend_by(skip);
                self.pos = span.end_position();
                Some(Err(error))
            },
        }
    }
}

impl<'text, K> std::iter::FusedIterator for Lexer<'text, K>
    where K: Tokenize,
{}

////////////////////////////////////////////////////////////////////////////////
// Tokenize
////////////////////////////////////////////////////////////////////////////////
/// Trait for parsing a value from a string prefix.
pub trait Tokenize: Sized {
    /// The parse token type.
    type Token: PartialEq + Send + Sync + 'static;
    /// The parse error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Parses a value from the given string. When the parse success, the
    /// length of the consumed text should be returned. When the parse fails,
    /// the length of the text to skip before resuming should be returned. If no
    /// further progress is possible, 0 should be returned instead.
    fn parse_token<'text>(&mut self, text: &'text str)
        -> Result<(Self::Token, Pos), (Self::Error, Pos)>;
}
