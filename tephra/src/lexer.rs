////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::position::ColumnMetrics;
use crate::position::LineEnding;
use crate::position::Pos;
use crate::result::SourceDisplay;
use crate::result::SourceSpan;
use crate::result::Highlight;
use crate::span::Span;

// External library imports.
use tracing::Level;
use tracing::event;

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

    /// Parses a token from the given string.
    fn scan<'text>(
        &mut self,
        source: &'text str,
        base: Pos,
        matrics: ColumnMetrics)
        -> Option<(Self::Token, Pos)>;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc> where Sc: Scanner {
    /// The source text to tokenize.
    source: &'text str,
    /// The internal user-defined Scanner.
    scanner: Sc,
    /// The token inclusion filter. Any token for which this returns false will
    /// be skipped automatically.
    filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>,

    /// The position of the start of the last lexed token.
    token_start: Pos,
    /// The position of the start of the current parse sequence.
    parse_start: Pos,
    /// The end position of the lexer cursor for the parse and last lexed token.
    end: Pos,
    /// The current position of the lexer cursor.
    cursor: Pos,

    /// The next token to emit, its position, and the resulting scanner state.
    buffer: Option<(Sc::Token, Pos, Sc)>,

    /// The source column metrics.
    metrics: ColumnMetrics,
}

impl<'text, Sc> Lexer<'text, Sc>
    where Sc: Scanner,
{
    /// Constructs a new Lexer for the given text.
    pub fn new(scanner: Sc, source: &'text str) -> Self {
        Lexer {
            source,
            scanner,
            filter: None,
            token_start: Pos::ZERO,
            parse_start: Pos::ZERO,
            end: Pos::ZERO,
            cursor: Pos::ZERO,
            buffer: None,
            metrics: Default::default(),
        }
    }

    /// Sets the column metrics for the Lexer.
    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Sets the line ending style for the Lexer.
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.metrics.line_ending = line_ending;
        self
    }

    /// Sets the tab width for the Lexer.
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.metrics.tab_width = tab_width;
        self
    }

    /// Returns true if there is no more text available to process.
    pub fn is_empty(&self) -> bool {
        self.cursor.byte >= self.source.len()
    }

    /// Returns the underlying source text.
    pub fn source(&self) -> &'text str {
        self.source
    }

    /// Returns the column metrics for the source.
    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    /// Returns the line ending for the source.
    pub fn line_ending(&self) -> LineEnding {
        self.metrics.line_ending
    }

    /// Returns the tab width for the source.
    pub fn tab_width(&self) -> u8 {
        self.metrics.tab_width
    }

    /// Returns the end position for the lexer's current parse.
    pub fn end_pos(&self) -> Pos {
        self.end
    }

    /// Returns the position of the lexer's cursor.
    pub fn cursor_pos(&self) -> Pos {
        self.cursor
    }


    /// Consumes the text from the `parse_start` of the current parse up to the
    /// current position. This ends the 'current parse' and prevents further
    /// spans from including any previously lexed text.
    fn consume_current(&mut self) {
        self.token_start = self.cursor;
        self.parse_start = self.cursor;
        self.end = self.cursor;
    }

    /// Returns an iterator over the lexer tokens together with their spans.
    pub fn iter_with_spans<'l>(&'l mut self)
        -> IterWithSpans<'text, 'l, Sc>
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
        self.cursor = self.end;
        self.buffer = None;
        self.filter.take()
    }

    /// Sets the token filter directly.
    pub fn set_filter(
        &mut self,
        filter: Option<Arc<dyn Fn(&Sc::Token) -> bool>>)
    {
        self.filter = filter;
        self.scan_to_buffer();
    }

    /// Scans to the next unfiltered token and buffers it. This method is
    /// idempotent.
    #[tracing::instrument(level = "debug")]
    fn scan_to_buffer(&mut self) {
        let mut scanner = self.scanner.clone();
        while self.cursor.byte < self.source.len() {
            match scanner.scan(self.source, self.cursor, self.metrics) {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    // Parsed a filtered token.
                    self.cursor = adv;
                },

                Some((token, adv)) => {
                    // Parsed a non-filtered token.
                    self.buffer = Some((token, adv, scanner));
                    break;
                },

                None => {
                    self.buffer = None; 
                    break;
                },
            }
        }
    }

    /// Returns the next token that would be returned by the `next` method
    /// without advancing the lexer position, assuming the lexer state is
    /// unchanged by the time `next` is called.
    #[tracing::instrument(level = "debug")]
    pub fn peek(&self) -> Option<Sc::Token> {
        if let Some((tok, _, _)) = self.buffer.as_ref() {
            Some(tok.clone())
        } else {
            // TODO: Make this more efficient.
            self.clone().next()
        }
    }

    #[tracing::instrument(level = "debug")]
    fn next_buffered(&mut self) -> Option<<Self as Iterator>::Item> {
        let (token, adv, scanner) = self.buffer.take()
            .expect("nonempty buffer");

        event!(Level::DEBUG, "token {:?}", token);

        self.token_start = self.cursor;
        self.end = adv;
        self.cursor = adv;
        self.scanner = scanner;
        self.scan_to_buffer();

        Some(token)
    }


    /// Creates a sublexer starting at the current lex position. The returned
    /// lexer will begin a new parse and advance past any filtered tokens.
    pub fn sublexer(&self) -> Self {
        let mut sub = self.clone();
        sub.consume_current();
        sub
    }

    /// Extends the receiver lexer to include the current parse span of the
    /// given lexer. (The given lexer's Scanner state will be discarded.)
    #[tracing::instrument(level = "debug")]
    pub fn join(mut self, other: Self) -> Self {
        // event!(Level::TRACE, "self\n{}", self);
        // event!(Level::TRACE, "other\n{}", other);

        if self.end < other.end {
            self.end = other.end;
            self.token_start = other.token_start;
        }

        if self.cursor < other.cursor {
            self.cursor = other.cursor;
        }

        self.scanner = other.scanner;
        self.buffer = other.buffer;
        // event!(Level::TRACE, "joined\n{}", self);
        self
    }
    
    /// Returns the span (excluding filtered text) of the token_start lexed
    /// token.
    pub fn token_span(&self) -> Span<'text> {
        Span::new_enclosing(self.source, self.token_start, self.end)
    }

    /// Returns the span (excluding filtered text) back to the token_start
    /// consumed  position.
    pub fn parse_span(&self) -> Span<'text> {
        Span::new_enclosing(self.source, self.parse_start, self.end)
    }

    /// Returns the cursor span (including filtered text) back to the
    /// token_start consumed position.
    pub fn parse_span_unfiltered(&self) -> Span<'text> {
        Span::new_enclosing(self.source, self.parse_start, self.cursor)
    }

    /// Returns the span of the end of the lexed text.
    pub fn end_span(&self) -> Span<'text> {
        Span::new_at(self.source, self.end)
    }

    /// Returns the span of the lexer cursor.
    pub fn cursor_span(&self) -> Span<'text> {
        Span::new_at(self.source, self.cursor)
    }
}

impl<'text, Sc> Iterator for Lexer<'text, Sc>
    where Sc: Scanner,
{
    type Item = Sc::Token;
    
    #[tracing::instrument(level = "debug")]
    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor.byte < self.source.len() {
            event!(Level::TRACE, "buffer {:?}", self.buffer);
            if self.buffer.is_some() {
                let res = self.next_buffered();
                event!(Level::TRACE, "lexer {}", self);
                return res;
            }

            match self.scanner.scan(
                self.source,
                self.cursor, 
                self.metrics)
            {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    // Parsed a filtered token.
                    self.cursor = adv;
                    self.token_start = adv;
                },

                Some((token, adv)) => {
                    event!(Level::DEBUG, "token {:?}", token);
                    // Parsed a non-filtered token.
                    self.token_start = self.cursor;
                    self.end = adv;
                    self.cursor = adv;

                    if self.filter.is_some() {
                        self.scan_to_buffer();
                    }
                    event!(Level::TRACE, "lexer {}", self);
                    return Some(token);
                },

                None => {
                    self.token_start = self.cursor;
                    event!(Level::DEBUG, "lexer is empty");
                    return None;
                },
            }
        }
        None
    }
}

impl<'text, Sc> Debug for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("source_len", &self.source.len())
            .field("scanner", &self.scanner)
            .field("filter_set", &self.filter.is_some())
            .field("token_start", &self.token_start)
            .field("parse_start", &self.parse_start)
            .field("end", &self.end)
            .field("cursor", &self.cursor)
            .field("buffer", &self.buffer)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl<'text, Sc> PartialEq for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source &&
        self.scanner == other.scanner &&
        self.filter.is_some() == other.filter.is_some() &&
        self.token_start == other.token_start &&
        self.parse_start == other.parse_start &&
        self.end == other.end && 
        self.cursor == other.cursor &&
        self.buffer == other.buffer &&
        self.metrics == other.metrics
    }
}

impl<'text, Sc> Display for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_display = SourceDisplay::new("lexer state")
            .with_color(false)
            .with_note_type()
            .with_source_span(SourceSpan::new(
                    self.parse_span_unfiltered(),
                    self.metrics)
                .with_highlight(Highlight::new(self.token_span(),
                    format!("token ({})", self.token_span())))
                .with_highlight(Highlight::new(self.parse_span(),
                    format!("parse ({})", self.parse_span())))
                .with_highlight(Highlight::new(self.cursor_span(),
                    format!("cursor ({})", self.cursor_span()))));

        writeln!(f, "Scanner: {:?}", self.scanner)?;
        write!(f, "{}", source_display)
    }
}

////////////////////////////////////////////////////////////////////////////////
// IterWithSpans
////////////////////////////////////////////////////////////////////////////////
/// An iterator over lexer tokens together with their spans. Created by the
/// `Lexer::iter_with_spans` method.
#[derive(Debug)]
pub struct IterWithSpans<'text, 'l, Sc> 
    where Sc: Scanner,
{
    lexer: &'l mut Lexer<'text, Sc>
}

impl<'text, 'l, Sc> Iterator for IterWithSpans<'text, 'l, Sc>
    where Sc: Scanner,
{
    type Item = (Sc::Token, Span<'text>);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer
            .next()
            .map(|t| (t, self.lexer.token_span()))
    }
}