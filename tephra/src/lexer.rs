////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer definitions.
////////////////////////////////////////////////////////////////////////////////
#![allow(missing_docs)]

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

pub trait Scanner: Debug + Clone + PartialEq {
    /// The parse token type.
    type Token: Display + Debug + Clone + PartialEq + Send + Sync + 'static;

    fn scan(&mut self, source: SourceTextRef<'_>, base: Pos)
        -> Option<(Self::Token, Pos)>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScannerBuffer<Sc>
    where Sc: Scanner,
{
    peek_scanner: Sc,
    peek_start: Pos,
    peek_cursor: Pos,
    token: Sc::Token
}

////////////////////////////////////////////////////////////////////////////////
// Lexer
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct Lexer<'text, Sc>
    where Sc: Scanner,
{
    source_text: SourceTextRef<'text>,
    scanner: Sc,
    filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>,
    recover: Option<Recover<Sc::Token>>,
    buffer: Option<ScannerBuffer<Sc>>,
    parse_start: Pos,
    token_start: Pos,
    cursor: Pos,
}

impl<'text, Sc> Lexer<'text, Sc>
    where Sc: Scanner,
{
    // Constructors
    ////////////////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn new(scanner: Sc, source_text: SourceTextRef<'text>) -> Self {
        Self {
            source_text,
            scanner,
            filter: None,
            recover: None,
            buffer: None,
            parse_start: Pos::default(),
            token_start: Pos::default(),
            cursor: Pos::default(),
        }
    }

    #[must_use]
    pub fn with_column_metrics(mut self, metrics: ColumnMetrics) -> Self {
        *self.column_metrics_mut() = metrics;
        self
    }

    #[must_use]
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.column_metrics_mut().line_ending = line_ending;
        self
    }

    #[must_use]
    pub fn with_tab_width(mut self, tab_width: u8) -> Self {
        self.column_metrics_mut().tab_width = tab_width;
        self
    }

    #[must_use]
    pub fn with_filter(mut self, filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>)
        -> Self
    {
        let _ = self.set_filter(filter);
        self
    }

    // Accessors
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the underlying source text.
    pub fn source_text(&self) -> SourceTextRef<'text> {
        self.source_text
    }

    /// Returns the column metrics for the source text.
    pub fn column_metrics(&self) -> ColumnMetrics {
        self.source_text.column_metrics()
    }

    fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
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

    // TODO: Mention filtered tokens.
    pub fn is_empty(&self) -> bool {
        self.cursor.byte >= self.source_text.len()
    }

    pub fn is_empty_with_filter(&mut self) -> bool {
        self.buffer_next();
        self.cursor.byte >= self.source_text.len()
    }
    
    pub fn recover_state(&self) -> Option<&Recover<Sc::Token>> {
        self.recover.as_ref()
    }

    pub fn set_recover_state(&mut self, recover: Option<Recover<Sc::Token>>) {
        self.recover = recover;
    }

    pub fn filter(&self) -> Option<&Rc<dyn Fn(&Sc::Token) -> bool>> {
        self.filter.as_ref()
    }

    pub fn set_filter(
        &mut self,
        filter: Option<Rc<dyn Fn(&Sc::Token) -> bool>>)
        -> Option<Rc<dyn Fn(&Sc::Token) -> bool>>
    {
        let res = self.filter.take();
        self.filter = filter;
        self.buffer = None;
        res
    }

    // Spans
    ////////////////////////////////////////////////////////////////////////////
    pub fn start_sublex(&mut self) {
        self.parse_start = self.cursor;
        self.token_start = self.cursor;
    }

    #[must_use]
    pub fn into_sublexer(mut self) -> Self {
        self.start_sublex();
        self
    }

    pub fn token_span(&self) -> Span {
        Span::enclosing(self.token_start, self.cursor)
    }

    pub fn parse_span(&self) -> Span {
        Span::enclosing(self.parse_start, self.cursor)
    }

    pub fn cursor_pos(&self) -> Pos {
        self.cursor
    }

    pub fn peek_token_span(&self) -> Option<Span> {
        self.buffer
            .as_ref()
            .and_then(|buf| if buf.peek_start == buf.peek_cursor {
                None
            } else {
                Some(Span::enclosing(buf.peek_start, buf.peek_cursor))
            })
    }

    pub fn peek_parse_span(&self) -> Option<Span> {
        self.buffer
            .as_ref()
            .map(|buf| if buf.peek_start == self.cursor {
                Span::enclosing(self.parse_start, buf.peek_cursor)
            } else {
                Span::enclosing(self.parse_start, self.cursor)
            })
    }

    pub fn peek_cursor_pos(&self) -> Option<Pos> {
        self.buffer
            .as_ref()
            .map(|buf| buf.peek_cursor)
    }

    // Peeking
    ////////////////////////////////////////////////////////////////////////////
    fn buffer_next(&mut self) {
        if self.buffer.is_some() { return; }

        let mut peek_scanner = self.scanner.clone();
        let mut peek_cursor = self.cursor;
        while let Some((tok, adv)) = peek_scanner
            .scan(self.source_text, peek_cursor)
        {
            if self.filter.as_ref().map_or(false, |f| !(f)(&tok)) {
                // Found a filtered token.
                peek_cursor = adv;
            } else {
                // Found a non-filtered token.
                self.buffer = Some(ScannerBuffer {
                    peek_scanner,
                    peek_start: peek_cursor,
                    peek_cursor: adv,
                    token: tok,
                });
                break;
            }
        }
    }

    pub fn peek(&mut self) -> Option<Sc::Token> {
        if self.cursor.byte >= self.source_text.len() {
            return None;
        }
        self.buffer_next();
        self.buffer
            .as_ref()
            .map(|buf| buf.token.clone())
    }

    pub fn next_if<P>(&mut self, pred: P) -> Option<Sc::Token>
        where P: FnOnce(&Sc::Token) -> bool
    {
        match self.peek().as_ref().map(pred) {
            Some(true) => self.next(),
            _          => None
        }
    }

    pub fn next_if_eq(&mut self, expected: &Sc::Token) -> Option<Sc::Token> {
        if self.peek().as_ref() == Some(expected) { self.next() } else { None }
    }

    // Advancing
    ////////////////////////////////////////////////////////////////////////////
    fn next_nonfiltered(&mut self) -> Option<Sc::Token> {
        if self.cursor.byte >= self.source_text.len() {
            return None;
        }
        if let Some(buf) = self.buffer.take() {
            self.scanner = buf.peek_scanner;
            self.token_start = buf.peek_start;
            if self.parse_start == self.cursor {
                self.parse_start = buf.peek_start;
            }
            self.cursor = buf.peek_cursor;
            return Some(buf.token);
        }

        let behind = self.parse_start == self.cursor;
        while let Some((tok, adv)) = self.scanner
            .scan(self.source_text, self.cursor)
        {
            if self.filter.as_ref().map_or(false, |f| !(f)(&tok)) {
                // Found a filtered token.
                self.cursor = adv;
            } else {
                // Found a non-filtered token.
                if behind {
                    self.parse_start = self.token_start;
                }
                self.token_start = self.cursor;
                self.cursor = adv;
                return Some(tok);
            }
        }
        None
    }

    pub fn advance_to_recover(&mut self) -> Result<Span, RecoverError> {
        if self.recover.is_none() {
            return Ok(Span::at(self.cursor));
        }

        let start_pos = self.cursor;
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
            Ok(Span::enclosing(start_pos, self.cursor))
        } else {
            Err(RecoverError)
        }
    }

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

    // Miscellaneous
    ////////////////////////////////////////////////////////////////////////////

    /// Returns an iterator over the lexer tokens together with their spans.
    pub fn iter_with_spans(&mut self) -> IterWithSpans<'text, '_, Sc>
        where Sc: Scanner
    {
        IterWithSpans { lexer: self }
    }
}

#[cfg(test)]
impl<'text, Sc> PartialEq for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn eq(&self, other: &Self) -> bool {
        self.scanner == other.scanner &&
        self.filter.is_some() == other.filter.is_some() &&
        self.recover.is_some() == other.recover.is_some() &&
        self.token_start == other.token_start &&
        self.parse_start == other.parse_start &&
        self.cursor == other.cursor &&
        self.buffer == other.buffer &&
        self.source_text == other.source_text
    }
}

impl<'text, Sc> Debug for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("parse_start", &self.parse_start)
            .field("token_start", &self.token_start)
            .field("cursor", &self.cursor)
            .field("buffer", &self.buffer)
            .field("scanner", &self.scanner)
            .field("filter", &self.filter.is_some())
            .field("recover", &self.recover.is_some())
            .field("source_text", &self.source_text)
            .finish()
    }
}

impl<'text, Sc> Display for Lexer<'text, Sc>
    where Sc: Scanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut spans = SpanDisplay::new(
                self.source_text,
                Span::enclosing(self.parse_start, self.cursor))
            .with_highlight(Highlight::new(self.token_span(),
                format!("token ({})", self.token_span())))
            .with_highlight(Highlight::new(self.parse_span(),
                format!("parse ({})", self.parse_span())))
            .with_highlight(Highlight::new(Span::at(self.cursor),
                format!("cursor ({}), scanner: {:?}",
                    Span::at(self.cursor),
                    self.scanner)));
        if let Some(span) = self.peek_token_span() {
            spans = spans.with_highlight(Highlight::new(span,
                format!("peek ({})", span)));
        }
        let source_display = CodeDisplay::new("Lexer")
            .with_color(true)
            .with_note_type()
            .with_span_display(spans);

        source_display.write(f, self.source_text)
    }
}

impl<'text, Sc> Iterator for Lexer<'text, Sc>
    where Sc: Scanner,
{
    type Item = Sc::Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.next_nonfiltered()
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
