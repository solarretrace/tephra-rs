////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer tests.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::any;
use crate::both;
use crate::one;
use crate::seq;
use crate::spanned;
use crate::text;

// External library imports.
use pretty_assertions::assert_eq;
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
use tephra::SourceText;
use tephra::SourceTextRef;
use tephra::Span;
use tephra::SpanDisplay;
use tephra::Spanned;


////////////////////////////////////////////////////////////////////////////////
// Abc scanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AbcToken {
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
pub struct Abc(Option<AbcToken>);

impl Abc {
    pub fn new() -> Self {
        Abc(None)
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

    fn scan<'text>(&mut self, source: SourceTextRef<'text>, base: Pos)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source.as_ref()[base.byte..];
        let metrics = source.column_metrics();

        if text.starts_with(',') {
            self.0 = Some(AbcToken::Comma);
            Some((
                AbcToken::Comma,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with(';') {
            self.0 = Some(AbcToken::Semicolon);
            Some((
                AbcToken::Semicolon,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with(']') {
            self.0 = Some(AbcToken::CloseBracket);
            Some((
                AbcToken::CloseBracket,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('[') {
            self.0 = Some(AbcToken::OpenBracket);
            Some((
                AbcToken::OpenBracket,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('a') {
            self.0 = Some(AbcToken::A);
            Some((
                AbcToken::A,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('b') {
            self.0 = Some(AbcToken::B);
            Some((
                AbcToken::B,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('c') {
            self.0 = Some(AbcToken::C);
            Some((
                AbcToken::C,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with("d") {
            self.0 = Some(AbcToken::D);
            Some((
                AbcToken::D,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))
            
        } else if text.starts_with(char::is_whitespace) {
            self.0 = Some(AbcToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            
            let substr_len = text.len() - rest.len();
            let substr = &source.as_ref()[0.. base.byte + substr_len];
            Some((AbcToken::Ws, metrics.end_position(substr, base)))
        } else if text.len() > 0 {
            self.0 = Some(AbcToken::Invalid);
            Some((
                AbcToken::Invalid,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))
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
pub enum Pattern<'text> {
    Abc(Spanned<&'text str>),
    Bxx(Spanned<&'text str>),
    Xyc(Spanned<&'text str>),
}

pub fn pattern<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, Pattern<'text>>
{
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
    let ctx = ctx.push(std::rc::Rc::new(move |e| {
        let parse_span = e.parse_span();
        let e = e.into_owned();
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

pub fn abc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;
    seq(&[A, B, C])
        (lexer, ctx)
        .map_value(|v| (v[0], v[1], v[2]))
}

pub fn bxx<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
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

pub fn xyc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
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
pub struct ParsePatternError {
    pub parse_span: Option<Span>,
    pub token_span: Option<Span>,
}

impl ParsePatternError {
    /// Converts the error into a `SourceError` attached to the given
    /// `SourceText`.
    pub fn into_source_error<'text>(self, source_text: SourceTextRef<'text>)
        -> SourceErrorRef<'text>
    {
        let mut se = SourceError::new(source_text, "expected pattern");
        if let Some(span) = self.parse_span {
            se = se
                .with_span_display(SpanDisplay::new_error_highlight(
                    source_text,
                    self.token_span.map(|t| span.enclose(t)).unwrap_or(span),
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
    fn parse_span(&self) -> Option<Span> {
        self.parse_span
    }

    fn into_source_error<'text>(self: Box<Self>, source_text: SourceTextRef<'text>)
        -> SourceErrorRef<'text>
    {
        Self::into_source_error(*self, source_text)
    }

    fn into_owned(self: Box<Self> ) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// Misc tests
////////////////////////////////////////////////////////////////////////////////

/// Tests Abc token lexing & filtering.
#[test]
fn abc_tokens() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "a b\nc d";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (
            lex.0,
            format!("{:?} ({})", source.clipped(lex.1).as_str(), lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
        (A,   "\"a\" (0:0-0:1, bytes 0-1)".to_string()),
        (B,   "\"b\" (0:2-0:3, bytes 2-3)".to_string()),
        (C,   "\"c\" (1:0-1:1, bytes 4-5)".to_string()),
        (D,   "\"d\" (1:2-1:3, bytes 6-7)".to_string()),
    ];

    assert_eq!(actual, expected);
}

/// Parses a `Pattern::Abc`.
#[test]
fn abc_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Bxx`.
#[test]
fn bxx_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "baa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Bxx(Spanned {
        value: "baa",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Xyc`.
#[test]
fn xyc_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "bac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "bac",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Ensures that a failure encountered after initial newline & whitespace
/// doesn't include that whitespace in the error message.
#[test]
fn initial_newline_ws_skip() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "\n    aaa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = pattern
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected pattern
 --> (1:0-1:7, bytes 1-8)
  | 
1 |     aaa
  |     ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
