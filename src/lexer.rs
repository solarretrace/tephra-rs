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
use crate::span::SpanPosition;


////////////////////////////////////////////////////////////////////////////////
// Lexeme
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text associated with a lexed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lexeme<'text, V> {
    /// The parsed token.
    pub value: V,
    /// The span of the parsed text.
    pub span: Span<'text>
}

impl<'text, V> Lexeme<'text, V> {
    /// Returns true if the token was parsed from whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.span.text().chars().all(char::is_whitespace)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'text, T> {
    text: &'text str,
    pos: SpanPosition,
    inner: T,
}

impl<'text, T> Lexer<'text, T> {
    /// Constructs a new Lexer for the given text and span newlines.
    pub fn new(inner: T, text: &'text str) -> Self {
        Lexer {
            text,
            pos: SpanPosition::ZERO,
            inner,
        }
    }
}

impl<'text, T> Iterator for Lexer<'text, T>
    where T: Tokenize,
{
    type Item = Result<Lexeme<'text, T::Token>, T::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.byte >= self.text.len() {
            return None;
        }

        match self.inner.parse_token(&self.text[self.pos.byte..]) {
            Ok((value, skip)) => {
                let mut span = Span::new_from(self.pos, self.text);
                span.extend_by(skip);
                self.pos = span.end_position();
                Some(Ok(Lexeme { value, span }))
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

impl<'text, T> std::iter::FusedIterator for Lexer<'text, T>
    where T: Tokenize,
{}

////////////////////////////////////////////////////////////////////////////////
// Tokenize
////////////////////////////////////////////////////////////////////////////////
/// Trait for parsing a value from a string prefix.
pub trait Tokenize: Sized {
    /// The parse token type.
    type Token: Send + Sync + 'static;
    /// The parse error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Parses a value from the given string. When the parse success, the
    /// length of the consumed text should be returned. When the parse fails,
    /// the length of the text to skip before resuming should be returned. If no
    /// further progress is possible, 0 should be returned instead.
    fn parse_token<'text>(&mut self, text: &'text str)
        -> Result<(Self::Token, SpanPosition), (Self::Error, SpanPosition)>;
}
