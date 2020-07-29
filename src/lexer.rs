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
    source: &'text str,
    scanner: Sc,
    newline: Nl,
    filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>,
    full: Pos,
    start: Pos,
    last_full: Pos,
    last: Pos,
    end: Pos,
    start_fixed: bool,
    // TODO: Push/lookahead buffer?
}

impl<'text, Sc, Nl> Lexer<'text, Sc, Nl> where Sc: Scanner {
    /// Constructs a new Lexer for the given text and span newlines.
    pub fn new(scanner: Sc, source: &'text str, newline: Nl) -> Self {
        Lexer {
            source,
            scanner,
            newline,
            filter: None,
            full: Pos::ZERO,
            start: Pos::ZERO,
            last_full: Pos::ZERO,
            last: Pos::ZERO,
            end: Pos::ZERO,
            start_fixed: false,
        }
    }

    /// Returns an iterator over the lexer tokens together with their spans.
    pub fn iter_with_spans<'l>(&'l mut self)
        -> IterWithSpans<'text, 'l, Sc, Nl>
        where Sc: Scanner
    {
        IterWithSpans { lexer: self }
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

    /// Returns the full underlying source text.
    pub fn source(&self) -> &'text str {
        self.source
    }

    /// Returns the current lexer position.
    pub fn current_pos(&self) -> Pos {
        self.end
    }

    /// Consumes text from the start of the lexed span up to the current
    /// position.
    pub fn consume_current(&mut self) {
        self.full = self.end;
        self.start = self.end;
    }

    /// Resets any lexed text back to the last consumed position.
    pub fn reset(&mut self) {
        self.start = self.full;
        self.end = self.full;
    }

    /// Returns the full span (including filtered text) back to the last
    /// consumed position.
    pub fn full_span(&self) -> Span<'text, Nl> {
        Span::new_enclosing(self.full, self.end, self.source)
    }

    /// Returns the span (excluding filtered text) back to the last consumed
    /// position.
    pub fn span(&self) -> Span<'text, Nl> {
        Span::new_enclosing(self.start, self.end, self.source)
    }

    /// Returns the span (excluding filtered text) of the last lexed token.
    pub fn last_full_span(&self) -> Span<'text, Nl> {
        Span::new_enclosing(self.last_full, self.end, self.source)
    }

    /// Returns the span (including filtered text) of the last lexed token.
    pub fn last_span(&self) -> Span<'text, Nl> {
        Span::new_enclosing(self.last, self.end, self.source)
    }

}

impl<'text, Sc, Nl> Iterator for Lexer<'text, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    type Item = Result<Sc::Token, Sc::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.end.byte < self.source.len() {

            match self.scanner
                .lex_prefix_token::<Nl>(&self.source[self.end.byte..])
            {
                Ok((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    self.last_full = self.end;
                    self.end += adv;
                    self.last = self.end;
                },

                Ok((token, adv)) => {
                    if !self.start_fixed {
                        self.start = self.end;
                        self.start_fixed = true;
                    }
                    self.last_full = self.end;
                    self.last = self.end;
                    self.end += adv;
                    return Some(Ok(token))
                },

                Err((error, adv)) => {
                    self.last_full = self.end;
                    self.end += adv;
                    if adv.is_zero() { self.end.byte = self.source.len() }
                    
                    return Some(Err(error))
                },
            }
        }
        None
    }
}

impl<'text, Sc, Nl> Debug for Lexer<'text, Sc, Nl> where Sc: Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<'text, Sc, Nl> PartialEq for Lexer<'text, Sc, Nl> where Sc: Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source &&
        self.full == other.full &&
        self.start == other.start &&
        self.last_full == other.last_full &&
        self.last == other.last &&
        self.end == other.end
    }
}

////////////////////////////////////////////////////////////////////////////////
// IterWithSpans
////////////////////////////////////////////////////////////////////////////////
/// An iterator over lexer tokens together with their spans. Created by the
/// `Lexer::iter_with_spans` method.
#[derive(Debug)]
pub struct IterWithSpans<'text, 'l, Sc, Nl> 
    where Sc: Scanner,
{
    lexer: &'l mut Lexer<'text, Sc, Nl>
}

impl<'text, 'l, Sc, Nl> Iterator for IterWithSpans<'text, 'l, Sc, Nl>
    where
        Sc: Scanner,
        Nl: NewLine,
{
    type Item = Result<(Sc::Token, Span<'text, Nl>), Sc::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|res| res.map(|t| {
             (t, self.lexer.last_span())
        }))
    }
}
