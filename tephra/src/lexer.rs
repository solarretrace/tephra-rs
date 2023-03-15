////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer definitions.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use tephra_error::Highlight;
use tephra_error::CodeDisplay;
use tephra_error::SpanDisplay;
use tephra_error::ParseError;
use tephra_error::ErrorSink;
use tephra_error::ErrorContext;
use tephra_span::ColumnMetrics;
use tephra_span::LineEnding;
use tephra_span::Pos;
use tephra_span::Span;
use tephra_span::SourceText;

// External library imports.
use tephra_tracing::Level;
use tephra_tracing::event;
use tephra_tracing::span;

// Standard library imports.
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;




////////////////////////////////////////////////////////////////////////////////
// Scanner
////////////////////////////////////////////////////////////////////////////////
/// Trait for parsing a value from a string prefix. Contains the lexer state for
/// a set of parseable tokens.
pub trait Scanner: Debug + Clone + PartialEq {
    /// The parse token type.
    type Token: Display + Debug + Clone + PartialEq + Send + Sync + 'static;

    /// Parses a token from the given source text.
    fn scan<'text>(&mut self, source: SourceText<'text>, base: Pos)
        -> Option<(Self::Token, Pos)>;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc> where Sc: Scanner {
    /// The source text to tokenize.
    source: SourceText<'text>,
    /// The token inclusion filter. Any token for which this returns false will
    /// be skipped automatically.
    filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>,
    /// The error sink.
    error_sink: Option<ErrorSink>,

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

    /// The internal user-defined Scanner.
    scanner: Sc,
}

impl<'text, Sc> Lexer<'text, Sc>
    where Sc: Scanner,
{
    /// Constructs a new Lexer for the given text.
    pub fn new(scanner: Sc, source: SourceText<'text>) -> Self {
        Lexer {
            source,
            filter: None,
            error_sink: None,
            token_start: Pos::ZERO,
            parse_start: Pos::ZERO,
            end: Pos::ZERO,
            cursor: Pos::ZERO,
            buffer: None,
            scanner,
        }
    }

    /// Sets the column metrics for the Lexer.
    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        *self.column_metrics_mut() = metrics;
        self
    }

    /// Sets the line ending style for the Lexer.
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.column_metrics_mut().line_ending = line_ending;
        self
    }

    /// Sets the tab width for the Lexer.
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.column_metrics_mut().tab_width = tab_width;
        self
    }

    /// Returns true if there is no more text available to process.
    pub fn is_empty(&self) -> bool {
        self.cursor.byte >= self.source.len()
    }

    /// Returns the underlying source text.
    pub fn source(&self) -> SourceText<'text> {
        self.source
    }

    /// Returns the column metrics for the source.
    pub fn column_metrics(&self) -> ColumnMetrics {
        self.source.column_metrics()
    }

    /// Returns the column metrics for the source.
    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        self.source.column_metrics_mut()
    }

    /// Returns the line ending for the source.
    pub fn line_ending(&self) -> LineEnding {
        self.column_metrics().line_ending
    }

    /// Returns the tab width for the source.
    pub fn tab_width(&self) -> u8 {
        self.column_metrics().tab_width
    }


    /// Sends a `ParseError` to the sink target, wrapping it in any available
    /// `ErrorContext`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send<'a, 't>(&'a self, parse_error: ParseError<'t>)
        -> Result<(), Box<dyn std::error::Error + 'a>>
    {
        match self.error_sink.as_ref() {
            None => {
                event!(Level::WARN, "A parse error sent to the error sink, \
                    but no error sink is configured:\n{}", parse_error);
                Ok(())
            },
            Some(sink) => sink.send(parse_error),
        }
    }

    /// Sends a `ParseError` to the sink target without wrapping it in any
    /// `ErrorContext`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send_direct<'a, 't>(&'a self, parse_error: ParseError<'t>) {
        match self.error_sink.as_ref() {
            None => {
                event!(Level::WARN, "A parse error sent to the error sink, \
                    but no error sink is configured:\n{}", parse_error);
            },
            Some(sink) => sink.send_direct(parse_error),
        }
    }


    /// Pushes a new `ErrorContext` onto the context stack, allowing any further
    /// `ParseError`s to be processed by them. 
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn push_context<'a>(&'a mut self, error_context: ErrorContext) {
        match self.error_sink.as_mut() {
            None => {
                event!(Level::WARN, "A context was pushed to the error sink, \
                    but no error sink is configured: {:?}", error_context);
            },
            Some(sink) => sink.push_context(error_context),
        }
    }

    /// Pops the top `ErrorContext` from the context stack.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn pop_context<'a>(&'a mut self) -> Option<ErrorContext> {
        match self.error_sink.as_mut() {
            None => {
                event!(Level::WARN, "A context was popped from the error sink, \
                    but no error sink is configured:");
                None
            },
            Some(sink) => sink.pop_context(),
        }
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
        self.set_filter(Some(Rc::new(filter)));
    }

    /// Returns the token filter, removing it from the lexer.
    pub fn take_filter(&mut self) -> Option<Rc<dyn Fn(&Sc::Token) -> bool>>
    {
        self.cursor = self.end;
        self.buffer = None;
        self.filter.take()
    }

    /// Sets the token filter directly.
    pub fn set_filter(
        &mut self,
        filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>)
    {
        self.filter = filter;
        self.scan_to_buffer();
    }

    /// Scans to the next unfiltered token and buffers it. This method is
    /// idempotent.
    fn scan_to_buffer(&mut self) {
        let _span = span!(Level::TRACE, "scan_to_buffer").entered();

        let mut scanner = self.scanner.clone();
        while self.cursor.byte < self.source.len() {
            match scanner.scan(self.source(), self.cursor) {
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
    pub fn peek(&self) -> Option<Sc::Token> {
        let _span = span!(Level::TRACE, "peek").entered();
        
        if let Some((tok, _, _)) = self.buffer.as_ref() {
            Some(tok.clone())
        } else {
            // TODO: Make this more efficient.
            self.clone().next()
        }
    }

    fn next_buffered(&mut self) -> Option<<Self as Iterator>::Item> {
        let _span = span!(Level::TRACE, "next_buffered").entered();

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

    /// Advances the lexer state up to the next instance of the given token.
    pub fn advance_to(&mut self, token: Sc::Token) {
        let _span = span!(Level::TRACE, "advance_to").entered();
        event!(Level::TRACE, "token={:?}, current: \n{}", token, self);
        while let Some(tok) = self.peek() {
            event!(Level::TRACE, "found {:?}", token);
            if tok == token { break; }
            let _ = self.next();
        }
        event!(Level::TRACE, "after: \n{}", self);
    }

    /// Advances the lexer state past the next instance of the given token.
    pub fn advance_past(&mut self, token: Sc::Token) {
        let _span = span!(Level::TRACE, "advance_past").entered();
        event!(Level::TRACE, "token={:?}, current: \n{}", token, self);
        while let Some(tok) = self.next() {
            event!(Level::TRACE, "found {:?}", token);
            if tok == token { break; }
        }
        event!(Level::TRACE, "after: \n{}", self);
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
    pub fn join(mut self, other: Self) -> Self {
        let _span = span!(Level::TRACE, "join").entered();

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
    pub fn token_span(&self) -> Span {
        Span::new_enclosing(self.token_start, self.end)
    }

    /// Returns the span (excluding filtered text) back to the token_start
    /// consumed  position.
    pub fn parse_span(&self) -> Span {
        Span::new_enclosing(self.parse_start, self.end)
    }

    /// Returns the cursor span (including filtered text) back to the
    /// token_start consumed position.
    pub fn parse_span_unfiltered(&self) -> Span {
        Span::new_enclosing(self.parse_start, self.cursor)
    }

    /// Returns the span of the end of the lexed text.
    pub fn end_span(&self) -> Span {
        Span::new_at(self.end)
    }

    /// Returns the span of the lexer cursor.
    pub fn cursor_span(&self) -> Span {
        Span::new_at(self.cursor)
    }
}

impl<'text, Sc> Iterator for Lexer<'text, Sc>
    where Sc: Scanner,
{
    type Item = Sc::Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        let _span = span!(Level::DEBUG, "next").entered();

        while self.cursor.byte < self.source.len() {
            event!(Level::TRACE, "buffer {:?}", self.buffer);
            if self.buffer.is_some() {
                let res = self.next_buffered();
                event!(Level::TRACE, "lexer {}", self);
                return res;
            }

            match self.scanner.scan(self.source, self.cursor) {
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
            .field("source", &self.source)
            .field("scanner", &self.scanner)
            .field("filter_set", &self.filter.is_some())
            .field("token_start", &self.token_start)
            .field("parse_start", &self.parse_start)
            .field("end", &self.end)
            .field("cursor", &self.cursor)
            .field("buffer", &self.buffer)
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
        self.buffer == other.buffer
    }
}

impl<'text, Sc> Display for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_display = CodeDisplay::new("lexer state")
            .with_color(true)
            .with_note_type()
            .with_span_display(SpanDisplay::new(
                    self.source,
                    self.parse_span_unfiltered())
                .with_highlight(Highlight::new(self.token_span(),
                    format!("token ({})", self.token_span())))
                .with_highlight(Highlight::new(self.parse_span(),
                    format!("parse ({})", self.parse_span())))
                .with_highlight(Highlight::new(self.cursor_span(),
                    format!("cursor ({})", self.cursor_span()))));

        writeln!(f, "Scanner: {:?}", self.scanner)?;
        source_display.write(f, self.source)
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
    type Item = (Sc::Token, Span);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer
            .next()
            .map(|t| (t, self.lexer.token_span()))
    }
}
