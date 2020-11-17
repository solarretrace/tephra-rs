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
use crate::position::ColumnMetrics;
use crate::position::Pos;
use crate::result::SourceDisplay;
use crate::result::SourceSpan;
use crate::result::Highlight;
use crate::span::Span;

// External library imports.
use tracing::Level;
use tracing::span;

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
    fn scan<'text, Cm>(&mut self, text: &'text str, metrics: Cm)
        -> Option<(Self::Token, Pos)>
        where Cm: ColumnMetrics;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc, Cm> where Sc: Scanner {
    /// The source text to tokenize.
    source: &'text str,
    /// The internal user-defined Scanner.
    scanner: Sc,
    /// The column metrics for handling of newlines and tabs.
    metrics: Cm,
    /// The token inclusion filter. Any token for which this returns false will
    /// be skipped automatically.
    filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>,

    /// The start position of the 'full span' for the current parse which
    /// includes filtered tokens.
    full: Pos,
    /// The start position of the 'span' for the current parse which
    /// excludes filtered tokens.
    start: Pos,

    /// The start position of the 'last full span' for the most recent token
    /// and any included filtered tokens.
    last_full: Pos,
    /// The start position of the 'last full span' for the most recent token.
    last: Pos,

    /// The current position of the lexer cursor.
    end: Pos,
    /// A flag indicating that a non-filtered token was lexed since the start
    /// of the current parse.
    start_fixed: bool,
}

impl<'text, Sc, Cm> Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    /// Constructs a new Lexer for the given text and column metrics.
    pub fn new(scanner: Sc, source: &'text str, metrics: Cm) -> Self {
        Lexer {
            source,
            scanner,
            metrics,
            filter: None,
            full: Pos::ZERO,
            start: Pos::ZERO,
            last_full: Pos::ZERO,
            last: Pos::ZERO,
            end: Pos::ZERO,
            start_fixed: false,
        }
    }

    /// Returns true if there is no more text available to process.
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

    /// Sets the token filter to the given function. Any token for which the
    /// filter returns `false` will be automatically skipped.
    pub fn set_filter_fn<F>(&mut self, filter: F) 
        where F: for<'a> Fn(&'a Sc::Token) -> bool + 'static
    {
        self.set_filter(Some(Arc::new(filter)));
    }

    /// Returns the token filter, removing it from the lexer.
    pub fn take_filter(&mut self) -> Option<Arc<dyn Fn(&Sc::Token) -> bool>>
    {
        self.filter.take()
    }

    /// Sets the token filter directly.
    pub fn set_filter(
        &mut self,
        filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>)
    {
        self.filter = filter;
    }

    /// Returns the full underlying source text.
    pub fn source(&self) -> &'text str {
        self.source
    }

    /// Returns the column metrics for the source.
    pub fn column_metrics(&self) -> Cm {
        self.metrics
    }

    /// Returns the start position for the lexer's current parse.
    pub fn start_pos(&self) -> Pos {
        self.start
    }

    /// Returns the end position for the lexer's current parse.
    pub fn end_pos(&self) -> Pos {
        self.end
    }

    /// Consumes the text from the start of the current parse up to the current
    /// position. This ends the 'current parse' and prevents further spans
    /// from including any previously lexed text.
    fn consume_current(&mut self) {
        self.full = self.end;
        self.start = self.end;
        self.start_fixed = false;
    }

    /// Creates a sublexer starting at the current lex position. The returned
    /// lexer will begin a new parse and advance past any filtered tokens.
    pub fn sublexer(&self) -> Self {
        let mut sub = self.clone();
        sub.filter_next();
        sub.consume_current();
        sub
    }

    /// Extends the receiver lexer to include the current parse span of the
    /// given lexer. (The given lexer's Scanner state will be discarded.)
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

    /// Returns the span of the remainder of the unlexed text.
    pub fn remaining_span(&self) -> Span<'text> {
        Span::new_enclosing(
            self.end,
            self.metrics.width(self.source),
            self.source)
    }

    /// Skips past any filtered tokens at the lex position.
    pub fn filter_next(&mut self) {
        let _ = self.next();
        // Move back to the start of last unfiltered token.
        self.end = self.last;
    }

    /// Returns the next token that would be returned by the `next` method
    /// without advancing the lexer position, assuming the lexer state is
    /// unchanged by the time `next` is called.
    pub fn peek(&self) -> Option<Sc::Token> {
        let span = span!(Level::DEBUG, "Lexer::peek");
        let _enter = span.enter();

        let mut scanner = self.scanner.clone();
        let mut end_byte = self.end.byte;
        while end_byte < self.source.len() {
            match scanner.scan(
                &self.source[end_byte..],
                self.metrics)
            {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    end_byte += adv.byte;
                },

                Some((token, _)) => return Some(token),
                None             => return None,
            }
        }
        None
    }
}

impl<'text, Sc, Cm> Iterator for Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    type Item = Sc::Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        let span = span!(Level::DEBUG, "Lexer::next");
        let _enter = span.enter();

        while self.end.byte < self.source.len() {
            match self.scanner.scan(
                &self.source[self.end.byte..],
                self.metrics)
            {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    // Parsed a filtered token.
                    self.last_full = self.end;
                    self.end += adv;
                    self.last = self.end;
                },

                Some((token, adv)) => {
                    // Parsed a non-filtered token.
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
                    return None;
                },
            }
        }
        None
    }
}

impl<'text, Sc, Cm> Debug for Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("source_len", &self.source.len())
            .field("scanner", &self.scanner)
            .field("metrics", &self.metrics)
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

impl<'text, Sc, Cm> PartialEq for Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
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

impl<'text, Sc, Cm> Display for Lexer<'text, Sc, Cm>
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f, "Source len: {:?}", self.source.len())?;
        // writeln!(f, "Full Span: {}", self.full_span())?;
        // writeln!(f, "Last Span (+filtered): {}", self.last_full_span())?;
        // writeln!(f, "Last Span: {}", self.last_span())?;
        // writeln!(f, "End Span: {}", self.end_span())?;
        // writeln!(f, "Remaining Span: {}", self.remaining_span())
        let source_display = SourceDisplay::new("lexer state")
            .with_color(false)
            .with_note_type()
            .with_source_span(SourceSpan::new(self.full_span(), self.metrics)
                .with_highlight(Highlight::new(self.last_span(), "last span"))
                .with_highlight(Highlight::new(self.last_full_span(), "last full span"))
                .with_highlight(Highlight::new(self.span(), "span")));

        write!(f, "{}", source_display)
    }
}

////////////////////////////////////////////////////////////////////////////////
// IterWithSpans
////////////////////////////////////////////////////////////////////////////////
/// An iterator over lexer tokens together with their spans. Created by the
/// `Lexer::iter_with_spans` method.
#[derive(Debug)]
pub struct IterWithSpans<'text, 'l, Sc, Cm> 
    where
        Sc: Scanner,
        Cm: ColumnMetrics,
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
