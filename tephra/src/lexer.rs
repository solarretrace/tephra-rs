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
use tephra_error::error::RecoverError;
use tephra_span::ColumnMetrics;
use tephra_span::LineEnding;
use tephra_span::Pos;
use tephra_span::Span;
use tephra_span::SourceTextRef;
use tephra_error::Recover;

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
    fn scan(&mut self, source: SourceTextRef<'_>, base: Pos)
        -> Option<(Self::Token, Pos)>;
}


////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
/// A lexical analyzer which lazily parses tokens from the source text.
#[derive(Clone)]
pub struct Lexer<'text, Sc> where Sc: Scanner {
    /// The source text to tokenize.
    source_text: SourceTextRef<'text>,
    /// The token inclusion filter. Any token for which this returns false will
    /// be skipped automatically.
    #[allow(clippy::type_complexity)]
    filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>,

    /// The position of the start of the last lexed token.
    token_start: Pos,
    /// The position of the start of the current parse sequence.
    parse_start: Pos,
    /// The end position of the lexer cursor for the parse and last lexed token.
    end: Pos,
    /// The current position of the lexer cursor.
    cursor: Pos,

    /// The error recovery type.
    recover: Option<Recover<Sc::Token>>,

    /// The next token to emit, its position, and the resulting scanner state.
    buffer: Option<(Sc::Token, Pos, Sc)>,
    /// The internal user-defined Scanner.
    scanner: Sc,
}

impl<'text, Sc> Lexer<'text, Sc>
    where Sc: Scanner,
{
    /// Constructs a new Lexer for the given text.
    #[must_use]
    pub fn new(scanner: Sc, source_text: SourceTextRef<'text>) -> Self {
        Lexer {
            source_text,
            filter: None,
            token_start: Pos::ZERO,
            parse_start: Pos::ZERO,
            end: Pos::ZERO,
            cursor: Pos::ZERO,
            buffer: None,
            recover: None,
            scanner,
        }
    }

    /// Sets the column metrics for the Lexer.
    #[must_use]
    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        *self.column_metrics_mut() = metrics;
        self
    }

    /// Sets the line ending style for the Lexer.
    #[must_use]
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.column_metrics_mut().line_ending = line_ending;
        self
    }

    /// Sets the tab width for the Lexer.
    #[must_use]
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.column_metrics_mut().tab_width = tab_width;
        self
    }

    /// Returns the underlying source text.
    pub fn source_text(&self) -> SourceTextRef<'text> {
        self.source_text
    }

    /// Returns the column metrics for the source text.
    pub fn column_metrics(&self) -> ColumnMetrics {
        self.source_text.column_metrics()
    }

    /// Returns the column metrics for the source.
    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        self.source_text.column_metrics_mut()
    }

    /// Returns the line ending for the source text.
    pub fn line_ending(&self) -> LineEnding {
        self.column_metrics().line_ending
    }

    /// Returns the tab width for the source text.
    pub fn tab_width(&self) -> u8 {
        self.column_metrics().tab_width
    }

    /// Returns true if there is no more text available to process.
    pub fn is_empty(&self) -> bool {
        // TODO: Replace with Option::is_some_and when stabilized.
        self.cursor.byte >= self.source_text.len()
            || match self.filter.as_ref()
        {
            None    => false,
            Some(_) => self.buffer.is_none(),
        }
    }

    /// Returns the start position for the lexer's current parse.
    pub fn start_pos(&self) -> Pos {
        self.parse_start
    }

    /// Returns the end position for the lexer's current parse.
    pub fn end_pos(&self) -> Pos {
        self.end
    }

    /// Returns the position of the lexer's cursor.
    pub fn cursor_pos(&self) -> Pos {
        self.cursor
    }

    /// Consumes the text from the `parse_start` of the current parse past any
    /// filtered tokens after the current position. This ends the 'current
    /// parse' and prevents further spans from including any previously lexed
    /// text.
    pub fn end_current_parse(&mut self) {
        self.scan_to_buffer();
        self.token_start = self.cursor;
        self.parse_start = self.cursor;
        self.end = self.cursor;
    }

    /// Creates a sublexer starting at the current lex position. The returned
    /// lexer will begin a new parse and advance past any filtered tokens.
    #[must_use]
    pub fn into_sublexer(mut self) -> Self {
        self.end_current_parse();
        self
    }

    /// Sets the token filter to the given function. Any token for which the
    /// filter returns `false` will be automatically skipped.
    pub fn set_filter_fn<F>(&mut self, filter: F) 
        where F: for<'a> Fn(&'a Sc::Token) -> bool + 'static
    {
        self.set_filter(Some(Rc::new(filter)));
    }

    /// Returns the token filter, removing it from the lexer.
    #[allow(clippy::type_complexity)]
    pub fn take_filter(&mut self) -> Option<Rc<dyn Fn(&Sc::Token) -> bool>>
    {
        self.cursor = self.end;
        self.buffer = None;
        self.filter.take()
    }

    /// Sets the token filter directly.
    #[allow(clippy::type_complexity)]
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

        let mut scanner = self.scanner.clone();
        while self.cursor.byte < self.source_text.len() {
            match scanner.scan(self.source_text(), self.cursor) {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    // Parsed a filtered token. If we have not yet started a
                    // parse, we advance the parse_start and the cursor.
                    if self.cursor == self.parse_start {
                        self.parse_start = adv;
                    }
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
        
        if let Some((tok, _, _)) = self.buffer.as_ref() {
            Some(tok.clone())
        } else {
            // TODO: Make this more efficient.
            self.clone().next()
        }
    }

    /// Retrieves the `next` token from the buffer if any token is buffered.
    fn next_buffered(&mut self) -> Option<<Self as Iterator>::Item> {

        match self.buffer.take() {
            None => None,
            Some((token, adv, scanner)) => {
                self.token_start = self.cursor;
                self.end = adv;
                self.cursor = adv;
                self.scanner = scanner;
                self.scan_to_buffer();

                Some(token)
            },
        }
    }

    /// Extends the receiver lexer to include the current parse span of the
    /// `other` lexer. (The `other` lexer's `Scanner` state will be discarded.)
    #[must_use]
    pub fn join(mut self, other: Self) -> Self {
        if self.end < other.end {
            self.end = other.end;
            self.token_start = other.token_start;
        }

        if self.cursor < other.cursor {
            self.cursor = other.cursor;
        }

        self.scanner = other.scanner;
        self.buffer = other.buffer;
        self
    }
    
    /// Returns the span (excluding filtered text) of the `token_start` lexed
    /// token.
    pub fn token_span(&self) -> Span {
        Span::new_enclosing(self.token_start, self.end)
    }

    /// Returns the span (excludinf filtered text) of the next available token.
    pub fn peek_token_span(&self) -> Option<Span> {
        if let Some((_, pos, _)) = self.buffer.as_ref() {
            Some(Span::new_enclosing(self.end, *pos))
        } else {
            self.clone()
                .iter_with_spans()
                .peekable()
                .peek()
                .map(|(_tok, span)| *span)
        }
    }

    /// Returns the span (excluding filtered text) from the start of the parse
    /// to the current position.
    pub fn parse_span(&self) -> Span {
        Span::new_enclosing(self.parse_start, self.end)
    }

    /// Returns the span (including filtered text) from the start of the parse
    /// to the current position.
    pub fn parse_span_unfiltered(&self) -> Span {
        Span::new_enclosing(self.parse_start, self.cursor)
    }

    /// Returns the span at the start of the current parse.
    pub fn start_span(&self) -> Span {
        Span::new_at(self.parse_start)
    }

    /// Returns the span at the end of the lexed text (the end of the current
    /// parse.)
    pub fn end_span(&self) -> Span {
        Span::new_at(self.end)
    }

    /// Returns the span at the lexer cursor.
    pub fn cursor_span(&self) -> Span {
        Span::new_at(self.cursor)
    }

    /// Returns an iterator over the lexer tokens together with their spans.
    pub fn iter_with_spans<'l>(&'l mut self)
        -> IterWithSpans<'text, 'l, Sc>
        where Sc: Scanner
    {
        IterWithSpans { lexer: self }
    }

    /// Returns the recover state used to indicate a token to advance to when a
    /// recoverable parse error occurs.
    pub fn recover_state(&self) -> Option<&Recover<Sc::Token>> {
        self.recover.as_ref()
    }

    /// Sets the recover state used to indicate a token to advance to when a
    /// recoverable parse error occurs.
    pub fn set_recover_state(&mut self, recover: Option<Recover<Sc::Token>>) {
        self.recover = recover;
    }

    /// Advances to the next token indicated by the current recover state.
    ///
    /// Returns the span of the skipped text, or an `UnexpectedTokenError` if the
    /// end-of-text is reached during recovery.
    pub fn advance_to_recover(&mut self) -> Result<Span, RecoverError> {
        if self.recover.is_none() {
            return Ok(self.cursor_span());
        }

        let start_pos = self.cursor_pos();
        let rec = self.recover
            .clone()
            .unwrap();

        let mut rec = rec.write()
            .expect("lock recover fn for writing");

        let mut token_found = false;
        {
            while let Some(token) = self.peek() {
                if rec(token)? {
                    token_found = true;
                    break;
                }
                let _ = self.next();
            }
        }

        if token_found {
            Ok(Span::new_enclosing(start_pos, self.cursor_pos()))
        } else {
            Err(RecoverError)
        }
    }


    /// Advances the lexer state up to the next token satisfying the given
    /// predicate. 
    ///
    /// Returns `true` if one of the given tokens was found, and `false` if the
    /// end of text was reached.
    pub fn advance_up_to<P>(&mut self, pred: P) -> bool
        where P: Fn(&Sc::Token) -> bool
    {
        while let Some(tok) = self.peek() {

            if pred(&tok) { return true; }
            let _ = self.next();
        }
        false
    }

    /// Advances the lexer state to after the next token satisfying the given
    /// predicate. 
    ///
    /// Returns `true` if one of the given tokens was found, and `false` if the
    /// end of text was reached.
    pub fn advance_to<P>(&mut self, pred: P) -> bool
        where P: Fn(&Sc::Token) -> bool
    {
        for tok in self.by_ref() {
            if pred(&tok) { return true; }
        }
        false
    }

    /// Advances the lexer state up to the next token satisfying the given
    /// predicate, with token filtering disabled.
    ///
    /// Returns `true` if one of the given tokens was found, and `false` if the
    /// end of text was reached.
    pub fn advance_up_to_unfiltered<P>(&mut self, pred: P) -> bool
        where P: Fn(&Sc::Token) -> bool
    {
        let filter = self.take_filter();
        let res = self.advance_up_to(pred);
        self.set_filter(filter);
        res
    }

    /// Advances the lexer state to after the next token satisfying the given
    /// predicate, with token filtering disabled.
    ///
    /// Returns `true` if one of the given tokens was found, and `false` if the
    /// end of text was reached.
    pub fn advance_to_unfiltered<P>(&mut self, pred: P) -> bool
        where P: Fn(&Sc::Token) -> bool
    {
        let filter = self.take_filter();
        let res = self.advance_to(pred);
        self.set_filter(filter);
        res
    }
}

impl<'text, Sc> Iterator for Lexer<'text, Sc>
    where Sc: Scanner,
{
    type Item = Sc::Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor.byte < self.source_text.len() {
            if self.buffer.is_some() {
                return self.next_buffered();
            }

            match self.scanner.scan(self.source_text, self.cursor) {
                Some((token, adv)) if self.filter
                    .as_ref()
                    .map_or(false, |f| !(f)(&token)) => 
                {
                    // Parsed a filtered token.
                    self.cursor = adv;
                    self.token_start = adv;
                },

                Some((token, adv)) => {
                    // Parsed a non-filtered token.
                    self.token_start = self.cursor;
                    self.end = adv;
                    self.cursor = adv;

                    if self.filter.is_some() {
                        self.scan_to_buffer();
                    }
                    return Some(token);
                },

                None => {
                    self.token_start = self.cursor;
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
            .field("source_text", &self.source_text)
            .field("scanner", &self.scanner)
            .field("filter_set", &self.filter.is_some())
            .field("token_start", &self.token_start)
            .field("parse_start", &self.parse_start)
            .field("end", &self.end)
            .field("cursor", &self.cursor)
            .field("recover", &self.recover.is_some())
            .field("buffer", &self.buffer)
            .finish()
    }
}


impl<'text, Sc> PartialEq for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn eq(&self, other: &Self) -> bool {
        self.source_text == other.source_text &&
        self.scanner == other.scanner &&
        self.filter.is_some() == other.filter.is_some() &&
        self.token_start == other.token_start &&
        self.parse_start == other.parse_start &&
        self.end == other.end && 
        self.cursor == other.cursor &&
        self.buffer == other.buffer &&
        self.recover.is_some() == other.recover.is_some()
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
                    self.source_text,
                    self.parse_span_unfiltered())
                .with_highlight(Highlight::new(self.token_span(),
                    format!("token ({})", self.token_span())))
                .with_highlight(Highlight::new(self.parse_span(),
                    format!("parse ({})", self.parse_span())))
                .with_highlight(Highlight::new(self.cursor_span(),
                    format!("cursor ({})", self.cursor_span()))));

        writeln!(f, "Scanner: {:?}", self.scanner)?;
        source_display.write(f, self.source_text)
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
