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
use crate::position::ColumnMetrics;
use crate::position::Pos;

// Standard library imports.
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;


////////////////////////////////////////////////////////////////////////////////
// Scanner
////////////////////////////////////////////////////////////////////////////////
/// Trait for parsing a value from a string prefix. Contains the lexer state for
/// a set of parseable tokens.
pub trait Scanner: Debug + Clone + PartialEq {
    /// The parse token type.
    type Token: Display + Debug + Clone + PartialEq + Send + Sync + 'static;

    /// Parses a token from the given string. When the parse success, the
    /// length of the consumed text should be returned. When the parse fails,
    /// the length of the text to skip before resuming should be returned. If no
    /// further progress is possible, 0 should be returned instead.
    fn scan<'text, Cm>(&mut self, text: &'text str)
        -> Option<(Self::Token, Pos)>
        where Cm: ColumnMetrics;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc, Cm> where Sc: Scanner {
    source: &'text str,
    scanner: Sc,
    newline: Cm,
    filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>,
    full: Pos,
    start: Pos,
    last_full: Pos,
    last: Pos,
    end: Pos,
    start_fixed: bool,
    // TODO: Push/lookahead buffer?
}

impl<'text, Sc, Cm> Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    /// Constructs a new Lexer for the given text and span newlines.
    pub fn new(scanner: Sc, source: &'text str, newline: Cm) -> Self {
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

    /// Returns true if the lexed text is empty.
    pub fn is_empty(&self) -> bool {
        self.end.byte >= self.source.len()
    }

    /// Returns an iterator over the lexer tokens together with their spans.
    pub fn iter_with_spans<'l>(&'l mut self)
        -> IterWithSpans<'text, 'l, Sc, Cm>
        where Sc: Scanner
    {
        IterWithSpans { lexer: self }
    }

    /// Sets the token filter to a given function. And token for which the
    /// filter returns `true` will be emitted to parsers.
    pub fn set_filter_fn<F>(&mut self, filter: F) 
        where F: for<'a> Fn(&'a Sc::Token) -> bool + 'static
    {
        self.filter = Some(Arc::new(filter));
    }

    /// Returns the token filter, removing it from the lexer.
    pub fn take_filter(&mut self) -> Option<Arc<dyn Fn(&Sc::Token) -> bool>>
    {
        self.filter.take()
    }

    /// Sets the token filter.
    pub fn set_filter(&mut self,
        filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>)
    {
        self.filter = filter;
    }

    /// Returns the full underlying source text.
    pub fn source(&self) -> &'text str {
        self.source
    }

    /// Returns the lexer's start position.
    pub fn start_pos(&self) -> Pos {
        self.end
    }

    /// Returns the current lexer position.
    pub fn end_pos(&self) -> Pos {
        self.end
    }

    /// Consumes text from the start of the lexed span up to the current
    /// position.
    fn consume_current(&mut self) {
        self.full = self.end;
        self.start = self.end;
    }

    /// Creates a sublexer starting at the current lex position.
    pub fn sublexer(&self) -> Self {
        let mut sub = self.clone();
        sub.next_filtered();
        sub.consume_current();
        sub
    }

    /// Joins the lexers together by unioning their full spans.
    /// The receiver's scanner state is retained.
    pub fn join(mut self, other: Self) -> Self {
        if self.end < other.end {
            self.end = other.end;
            self.last = other.last;
            self.last_full = other.last_full;
        }

        if self.full > other.full {
            self.full = other.full;
            self.start = other.start;
        }
        self
    }

    /// Resets any lexed text back to the last consumed position.
    pub fn reset(&mut self) {
        self.start = self.full;
        self.end = self.full;
    }

    /// Returns the full span (including filtered text) back to the last
    /// consumed position.
    pub fn full_span(&self) -> Span<'text> {
        Span::new_enclosing(self.full, self.end, self.source)
    }

    /// Returns the span (excluding filtered text) back to the last consumed
    /// position.
    pub fn span(&self) -> Span<'text> {
        Span::new_enclosing(self.start, self.end, self.source)
    }

    /// Returns the span (including filtered text) of the last lexed token.
    pub fn last_full_span(&self) -> Span<'text> {
        Span::new_enclosing(self.last_full, self.end, self.source)
    }

    /// Returns the span (excluding filtered text) of the last lexed token.
    pub fn last_span(&self) -> Span<'text> {
        Span::new_enclosing(self.last, self.end, self.source)
    }

    /// Returns the span of the end of the lexed text.
    pub fn end_span(&self) -> Span<'text> {
        Span::new_from(self.end, self.source)
    }

    /// Skips past any filtered tokens at the lex position.
    pub fn next_filtered(&mut self) {
        let _ = self.next();
        // Move back to the start of last unfiltered token.
        self.end = self.last;
    }

}

impl<'text, Sc, Cm> Iterator for Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    type Item = Sc::Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.end.byte < self.source.len() {
            match self.scanner.scan::<Cm>(&self.source[self.end.byte..]) {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    self.last_full = self.end;
                    self.end += adv;
                    self.last = self.end;
                },

                Some((token, adv)) => {
                    if !self.start_fixed {
                        self.start = self.end;
                        self.start_fixed = true;
                    }
                    self.last_full = self.end;
                    self.last = self.end;
                    self.end += adv;
                    return Some(token);
                },

                None => {
                    self.last_full = self.end;
                    // self.end += adv;
                    // if adv.is_zero() { self.end.byte = self.source.len() }
                    
                    return None;
                },
            }
        }
        None
    }
}

impl<'text, Sc, Cm> Debug for Lexer<'text, Sc, Cm> where Sc: Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("source_len", &self.source.len())
            .field("scanner", &self.scanner)
            .field("filter_set", &self.filter.is_some())
            .field("full", &self.full)
            .field("start", &self.start)
            .field("last_full", &self.last_full)
            .field("last", &self.last)
            .field("end", &self.end)
            .field("start_fixed", &self.start_fixed)
            .finish()
    }
}

impl<'text, Sc, Cm> PartialEq for Lexer<'text, Sc, Cm> where Sc: Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source &&
        self.scanner == other.scanner &&
        self.filter.is_some() == other.filter.is_some() &&
        self.full == other.full &&
        self.start == other.start &&
        self.last_full == other.last_full &&
        self.last == other.last &&
        self.end == other.end && 
        self.start_fixed == other.start_fixed
    }
}


////////////////////////////////////////////////////////////////////////////////
// IterWithSpans
////////////////////////////////////////////////////////////////////////////////
/// An iterator over lexer tokens together with their spans. Created by the
/// `Lexer::iter_with_spans` method.
#[derive(Debug)]
pub struct IterWithSpans<'text, 'l, Sc, Cm> 
    where Sc: Scanner,
{
    lexer: &'l mut Lexer<'text, Sc, Cm>
}

impl<'text, 'l, Sc, Cm> Iterator for IterWithSpans<'text, 'l, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    type Item = (Sc::Token, Span<'text>);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|t| {
             (t, self.lexer.last_span())
        })
    }
}
