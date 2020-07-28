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
use crate::span::NewLine;
use crate::span::Pos;

// Standard library imports.
use std::fmt::Debug;
use std::sync::Arc;


////////////////////////////////////////////////////////////////////////////////
// Scanner
////////////////////////////////////////////////////////////////////////////////
/// Trait for parsing a value from a string prefix. Contains the lexer state for
/// a set of parseable tokens.
pub trait Scanner: Debug + Clone + PartialEq {
    /// The parse token type.
    type Token: Debug + Clone + PartialEq + Send + Sync + 'static;
    /// The parse error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Parses a token from the given string. When the parse success, the
    /// length of the consumed text should be returned. When the parse fails,
    /// the length of the text to skip before resuming should be returned. If no
    /// further progress is possible, 0 should be returned instead.
    fn lex_prefix_token<'text, Nl>(&mut self, text: &'text str)
        -> Result<(Self::Token, Pos), (Self::Error, Pos)>
        where Nl: NewLine;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc, Nl> where Sc: Scanner {
    newline: Nl,
    source: &'text str,
    consumed: Pos,
    current: Pos,
    scanner: Sc,
    filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>,
    // TODO: Push/lookahead buffer?
}

impl<'text, Sc, Nl> Lexer<'text, Sc, Nl> where Sc: Scanner {
    /// Constructs a new Lexer for the given text and span newlines.
    pub fn new(scanner: Sc, source: &'text str, newline: Nl) -> Self {
        Lexer {
            newline,
            source,
            consumed: Pos::ZERO,
            current: Pos::ZERO,
            scanner,
            filter: None,
        }
    }

    /// Sets the token filter.
    pub fn set_filter<F>(&mut self, filter: F) 
        where F: for<'a> Fn(&'a Sc::Token) -> bool + 'static
    {
        self.filter = Some(Arc::new(filter));
    }

    /// Clears the token filter.
    pub fn clear_filters(&mut self) {
        self.filter = None;
    }

    /// Returns the current lexer position.
    pub fn current_pos(&self) -> Pos {
        self.current
    }

    /// Returns the full underlying source text.
    pub fn source(&self) -> &'text str {
        self.source
    }

    /// Resets the lexer position to the start position of the uncomsumed text.
    /// Note that this does not modify the scanner state or filters.
    pub fn reset(&mut self) {
        self.current = self.consumed
    }

    /// Returns the unconsumed span up to the current position.
    pub fn current_span(&self) -> Span<'text, Nl> {
        Span::new_enclosing(self.consumed, self.current, self.source)
    }

    /// Returns the span up to the current position and consumes it.
    pub fn consume_current_span(&mut self) -> Span<'text, Nl> {
        let span = Span::new_enclosing(
            self.consumed,
            self.current,
            self.source);
        self.consumed = self.current;
        span
    }
}

impl<'text, Sc, Nl> Iterator for Lexer<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    type Item = Result<Lexeme<'text, Sc::Token, Nl>, Sc::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current.byte < self.source.len() {

            match self.scanner
                .lex_prefix_token::<Nl>(&self.source[self.current.byte..])
            {
                Ok((token, skip)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    self.current += skip;
                },

                Ok((token, skip)) => {
                    let mut span = Span::new_from(self.current, self.source);
                    span.extend_by(skip);
                    self.current = span.end();
                    return Some(Ok(Lexeme { token, span }))
                },

                Err((error, skip)) => {
                    if skip.is_zero() { self.current.byte = self.source.len() }

                    let mut span: Span<'_, Nl> = Span::new_from(
                        self.current,
                        self.source);
                    span.extend_by(skip);
                    self.current = span.end();
                    return Some(Err(error))
                },
            }
        }
        None
    }
}

impl<'text, Sc, Nl> Debug for Lexer<'text, Sc, Nl> where Sc: Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl<'text, Sc, Nl> PartialEq for Lexer<'text, Sc, Nl> where Sc: Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source &&
        self.consumed == other.consumed &&
        self.current == other.current
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexeme
////////////////////////////////////////////////////////////////////////////////
/// A specific section of the source text associated with a lexed token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lexeme<'text, T, Nl> {
    /// The parsed token.
    token: T,
    /// The span of the parsed text.
    span: Span<'text, Nl>
}

impl<'text, T, Nl> Lexeme<'text, T, Nl>
    where
        T: PartialEq,
{
    /// Returns true if the token was parsed from whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.span.text().chars().all(char::is_whitespace)
    }

    /// Returns a reference to the lexed token.
    pub fn token(&self) -> &T {
        &self.token
    }

    /// Returns a reference to the lexed token's span.
    pub fn span(&self) -> &Span<'text, Nl> {
        &self.span
    }

    /// Consumes the lexeme and returns its span.
    pub fn into_span(self) -> Span<'text, Nl> {
        self.span
    }
}

impl<'text, T, Nl> PartialEq<T> for Lexeme<'text, T, Nl>
    where T: PartialEq
{
    fn eq(&self, other: &T) -> bool {
        self.token == *other
    }
}
