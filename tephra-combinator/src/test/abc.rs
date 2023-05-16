////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Test scanner & parser definitions.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::any;
use crate::both;
use crate::one;
use crate::seq;
use crate::spanned;
use crate::text;

// External library imports.
use tephra::Context;
use tephra::error::Found;
use tephra::error::SourceError;
use tephra::error::SourceErrorRef;
use tephra::error::UnexpectedTokenError;
use tephra::Lexer;
use tephra::ParseError;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Pos;
use tephra::Scanner;
use tephra::SourceTextRef;
use tephra::Span;
use tephra::SpanDisplay;
use tephra::Spanned;
use tephra_tracing::event;
use tephra_tracing::Level;
use tephra_tracing::span;


////////////////////////////////////////////////////////////////////////////////
// Abc scanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(in crate::test) enum AbcToken {
    A,
    B,
    C,
    D,
    Ws,
    Comma,
    Semicolon,
    OpenBracket,
    CloseBracket,
    Invalid,
}

impl AbcToken {
    fn is_pattern_token(&self) -> bool {
        use AbcToken::*;
        match self {
            A | B | C | D => true,
            _             => false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::test) struct Abc(Option<AbcToken>);

impl Abc {
    pub(in crate::test) fn new() -> Self {
        Self(None)
    }
}

impl std::fmt::Display for AbcToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AbcToken::*;
        match self {
            A            => write!(f, "'a'"),
            B            => write!(f, "'b'"),
            C            => write!(f, "'c'"),
            D            => write!(f, "'d'"),
            Ws           => write!(f, "whitespace"),
            Comma        => write!(f, "','"),
            Semicolon    => write!(f, "','"),
            OpenBracket  => write!(f, "'['"),
            CloseBracket => write!(f, "']'"),
            Invalid      => write!(f, "invalid token"),
        }
    }
}

impl Scanner for Abc {
    type Token = AbcToken;

    fn scan(&mut self, source: SourceTextRef<'_>, base: Pos)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source.as_ref()[base.byte..];
        let metrics = source.column_metrics();

        if text.starts_with(',') {
            self.0 = Some(AbcToken::Comma);
            Some((
                AbcToken::Comma,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with(';') {
            self.0 = Some(AbcToken::Semicolon);
            Some((
                AbcToken::Semicolon,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with(']') {
            self.0 = Some(AbcToken::CloseBracket);
            Some((
                AbcToken::CloseBracket,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('[') {
            self.0 = Some(AbcToken::OpenBracket);
            Some((
                AbcToken::OpenBracket,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('a') {
            self.0 = Some(AbcToken::A);
            Some((
                AbcToken::A,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('b') {
            self.0 = Some(AbcToken::B);
            Some((
                AbcToken::B,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('c') {
            self.0 = Some(AbcToken::C);
            Some((
                AbcToken::C,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('d') {
            self.0 = Some(AbcToken::D);
            Some((
                AbcToken::D,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))
            
        } else if text.starts_with(char::is_whitespace) {
            self.0 = Some(AbcToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            
            let substr_len = text.len() - rest.len();
            let substr = &source.as_ref()[0.. base.byte + substr_len];
            Some((AbcToken::Ws, metrics.end_position(substr, base)))
        } else if !text.is_empty() {
            self.0 = Some(AbcToken::Invalid);
            Some((
                AbcToken::Invalid,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))
        } else {
            self.0 = None;
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Abc grammar
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::test) enum Pattern<'text> {
    Abc(Spanned<&'text str>),
    Bxx(Spanned<&'text str>),
    Xyc(Spanned<&'text str>),
}

pub(in crate::test) fn pattern<'text>(
    mut lexer: Lexer<'text, Abc>,
    ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, Pattern<'text>>
{
    let _trace_span = span!(Level::DEBUG, "pattern").entered();

    // Optimization to shortcut when we know a pattern can't succeed:
    match lexer.peek() {
        Some(tok) if !tok.is_pattern_token() => {
            event!(Level::DEBUG, "non-pattern token found: ({:?})", tok);
            let parse_span = Some(lexer.parse_span());
            let token_span = Some(lexer.token_span());
            return Err(Box::new(ParsePatternError { parse_span, token_span }));
        },
        _ => (),
    }


    match spanned(text(abc))
        (lexer.clone(), ctx.clone())
        .map_value(Pattern::Abc)
    {
        Ok(succ) => { return Ok(succ); },
        _ => (),
    }

    match spanned(text(bxx))
        (lexer.clone(), ctx.clone())
        .map_value(Pattern::Bxx)
    {
        Ok(succ) => { return Ok(succ); },
        _ => (),
    }

    // Setup error context.
    let ctx = ctx.pushed(std::rc::Rc::new(move |e| {
        let parse_span = e.error_span();
        let e = e.into_error();
        // Collect the token span so that we can include it in the final span.
        // Non-pattern tokens are excluded.
        let token_span = e
            .downcast_ref::<UnexpectedTokenError<AbcToken>>()
            .and_then(|e| match e.found {
                Found::Token(t) if t.is_pattern_token()
                    => Some(e.token_span),
                _   => None,
            });

        Box::new(ParsePatternError { parse_span, token_span })
    }));

    spanned(text(xyc))
        (lexer, ctx.clone())
        .apply_context(ctx)
        .map_value(Pattern::Xyc)
}

pub(in crate::test) fn abc<'text>(
    lexer: Lexer<'text, Abc>,
    ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    let _trace_span = span!(Level::TRACE, "abc").entered();

    use AbcToken::*;
    seq(&[A, B, C])
        (lexer, ctx)
        .map_value(|v| (v[0], v[1], v[2]))
}

pub(in crate::test) fn bxx<'text>(
    lexer: Lexer<'text, Abc>,
    ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    let _trace_span = span!(Level::TRACE, "bxx").entered();

    use AbcToken::*;
    let ((x, y), succ) = both(
            one(B),
            any(&[A, B, C, D]))
        (lexer, ctx.clone())?
        .take_value();

    one(y)
        (succ.lexer, ctx)
        .map_value(|z| (x, y, z))
}

pub(in crate::test) fn xyc<'text>(
    lexer: Lexer<'text, Abc>,
    ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    let _trace_span = span!(Level::TRACE, "xyc").entered();

    use AbcToken::*;
    both(
        both(
            any(&[A, B, C, D]),
            any(&[A, B, C, D])),
        one(C))
        (lexer, ctx)
        .map_value(|((x, y), z)| (x, y, z))
}


////////////////////////////////////////////////////////////////////////////////
// ParsePatternError
////////////////////////////////////////////////////////////////////////////////
/// An error generated when a successful parse does not consume as much text as
/// required.
#[derive(Debug, Clone)]
pub(in crate::test) struct ParsePatternError {
    pub(in crate::test) parse_span: Option<Span>,
    pub(in crate::test) token_span: Option<Span>,
}

impl ParsePatternError {
    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    pub(in crate::test) fn into_source_error(
        self,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        let mut se = SourceError::new(source_text, "expected pattern");
        if let Some(span) = self.parse_span {
            se = se
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    self.token_span.map_or(span, |t| span.enclose(t)),
                    "expected 'ABC', 'BXX', or 'XYC' pattern"))
        };
        se.with_cause(Box::new(self))
    }
}

impl std::fmt::Display for ParsePatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected pattern")
    }
}

impl std::error::Error for ParsePatternError {}

impl ParseError for ParsePatternError {
    fn error_span(&self) -> Option<Span> {
        self.parse_span
    }

    fn into_source_error(
        self: Box<Self>,
        source_text: SourceTextRef<'_>)
        -> SourceErrorRef<'_>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_error(self: Box<Self>)
        -> Box<dyn std::error::Error + Send + Sync + 'static>
    {
        self
    }
}

