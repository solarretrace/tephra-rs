////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer tests.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=[test_name]=TRACE cargo test test_name -- --nocapture


// Local imports.
use crate::combinator::both;
use crate::combinator::one;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::position::Pos;
use crate::result::ParseResultExt as _;

// External library imports.
use pretty_assertions::assert_eq;
use test_log::test;


////////////////////////////////////////////////////////////////////////////////
// Token parser.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum TestToken {
    Aa,
    A,
    B,
    Def,
    Ws,
}

impl std::fmt::Display for TestToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TestToken::*;
        match self {
            Aa  => write!(f, "'aa'"),
            A   => write!(f, "'a'"),
            B   => write!(f, "'b'"),
            Def => write!(f, "'def'"),
            Ws  => write!(f, "whitespace"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Test(Option<TestToken>);

impl Test {
    pub fn new() -> Self {
        Test(None)
    }
}

impl Scanner for Test {
    type Token = TestToken;

    fn scan<'text>(
        &mut self,
        source: &'text str,
        base: Pos,
        metrics: ColumnMetrics)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source[base.byte..];

        if text.starts_with("aa") {
            self.0 = Some(TestToken::Aa);
            Some((
                TestToken::Aa,
                metrics.end_position(&source[..base.byte + 2], base)))

        } else if text.starts_with('a') {
            self.0 = Some(TestToken::A);
            Some((
                TestToken::A,
                metrics.end_position(&source[..base.byte + 1], base)))

        } else if text.starts_with('b') {
            self.0 = Some(TestToken::B);
            Some((
                TestToken::B,
                metrics.end_position(&source[..base.byte + 1], base)))

        } else if text.starts_with("def") {
            self.0 = Some(TestToken::Def);
            Some((
                TestToken::Def,
                metrics.end_position(&source[..base.byte + 3], base)))
            
        } else {
            self.0 = Some(TestToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            if rest.len() < text.len() {
                let substr_len = text.len() - rest.len();
                let substr = &source[0.. base.byte + substr_len];
                Some((TestToken::Ws, metrics.end_position(substr, base)))
            } else {
                self.0 = None;
                None
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////


/// Performs size checks.
#[allow(unused_qualifications)]
#[test]
#[tracing::instrument]
fn size_checks() {
    use std::mem::size_of;
    assert_eq!(168, size_of::<crate::lexer::Lexer<'_, Test>>(), "Lexer");
}

/// Tests `Lexer::new`.
#[test]
#[tracing::instrument]
fn empty() {
    let text = "";
    let mut lexer = Lexer::new(Test::new(), text);

    assert_eq!(
        lexer.next(),
        None);
}

/// Tests `Lexer::next`.
#[test]
#[tracing::instrument]
fn simple() {
    use TestToken::*;
    let text = "aa b";
    let mut lexer = Lexer::new(Test::new(), text);

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(lexer.next(), Some(Ws));
    assert_eq!(lexer.next(), Some(B));
}


/// Tests `Lexer::peek`.
#[test]
#[tracing::instrument]
fn simple_peek() {
    use TestToken::*;
    let text = "aa b";
    let mut lexer = Lexer::new(Test::new(), text);

    assert_eq!(lexer.peek(), Some(Aa));
    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(lexer.peek(), Some(Ws));
    assert_eq!(lexer.next(), Some(Ws));
    assert_eq!(lexer.peek(), Some(B));
    assert_eq!(lexer.next(), Some(B));
    assert_eq!(lexer.peek(), None);
    assert_eq!(lexer.next(), None);
}

/// Tests `Lexer::iter_with_spans`.
#[test]
#[tracing::instrument]
fn simple_iter() {
    use TestToken::*;
    let text = "aa b";
    let mut lexer = Lexer::new(Test::new(), text);

    assert_eq!(
        lexer
            .iter_with_spans()
            .map(|lex| (lex.0, format!("{:?}", lex.1)))
            .collect::<Vec<_>>(),
        vec![
            (Aa, "\"aa\" (0:0-0:2, bytes 0-2)".to_string()),
            (Ws, "\" \" (0:2-0:3, bytes 2-3)".to_string()),
            (B,  "\"b\" (0:3-0:4, bytes 3-4)".to_string()),
        ]);
}


/// Tests `Lexer`'s auto-filtering capability.
#[test]
#[tracing::instrument]
fn auto_filter() {
    use TestToken::*;
    let text = "aaaabaaaab";
    let mut lexer = Lexer::new(Test::new(), text);
    lexer.set_filter_fn(|tok| *tok != Aa);

    assert_eq!(lexer.peek(), Some(B));

    let f = lexer.take_filter();

    assert_eq!(lexer.peek(), Some(Aa));

    lexer.set_filter(f);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (lex.0, format!("{:?}", lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
            (B,   "\"b\" (0:4-0:5, bytes 4-5)".to_string()),
            (B,   "\"b\" (0:9-0:10, bytes 9-10)".to_string()),
        ];

    // for (i, act) in actual.iter().enumerate() {
    //     println!("{:?}", act);
    //     println!("{:?}", expected[i]);
    //     println!();
    // }

    assert_eq!(actual, expected);
}


/// Tests `Lexer` with whitespace filter.
#[test]
#[tracing::instrument]
fn whitespace_filter() {
    use TestToken::*;
    let text = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test::new(), text);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (lex.0, format!("{:?}", lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
            (Aa,  "\"aa\" (0:0-0:2, bytes 0-2)".to_string()),
            (B,   "\"b\" (0:3-0:4, bytes 3-4)".to_string()),
            (B,   "\"b\" (1:0-1:1, bytes 6-7)".to_string()),
            (Def, "\"def\" (1:1-1:4, bytes 7-10)".to_string()),
            (Aa,  "\"aa\" (2:1-2:3, bytes 12-14)".to_string()),
            (A,   "\"a\" (2:3-2:4, bytes 14-15)".to_string()),
        ];

    // for (i, act) in actual.iter().enumerate() {
    //     println!("{:?}", act);
    //     println!("{:?}", expected[i]);
    //     println!();
    // }

    assert_eq!(actual, expected);
}


/// Tests `both` with whitespace filter.
#[test]
#[tracing::instrument]
fn atma_issue_1_both_whitespace_filter() {
    use TestToken::*;
    let input = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test::new(), input);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            text(one(Aa)),
            text(one(B)))
        (lexer)
        .finish()
        .unwrap();

    let expected = ("aa", "b");

    assert_eq!(actual, expected);
}

/// Tests `Lexer` display output formatting.
#[test]
#[tracing::instrument]
fn display_formatting() {
    use TestToken::*;
    let text = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test::new(), text);
    lexer.set_filter_fn(|tok| *tok != Ws);

    assert_eq!(format!("{}", lexer), "\
Scanner: Test(None)
note: lexer state
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | \\ token (0:0, byte 0)
  | \\ parse (0:0, byte 0)
  | \\ cursor (0:0, byte 0)
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(Aa))
note: lexer state
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | -- token (0:0-0:2, bytes 0-2)
  | -- parse (0:0-0:2, bytes 0-2)
  |    \\ cursor (0:3, byte 3)
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(B))
note: lexer state
 --> (0:0-1:4, bytes 0-10)
  | 
0 | aa b 
  |    - token (0:3-0:4, bytes 3-4)
  | ---- parse (0:0-0:4, bytes 0-4)
1 | bdef
  | \\ cursor (1:0, byte 6)
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(B))
note: lexer state
 --> (0:0-1:4, bytes 0-10)
  | 
0 | / aa b 
1 | | bdef
  | | - token (1:0-1:1, bytes 6-7)
  | |_^ parse (0:0-1:1, bytes 0-7)
  |    \\ cursor (1:1, byte 7)
");

    assert_eq!(lexer.next(), Some(Def));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(Def))
note: lexer state
 --> (0:0-2:4, bytes 0-15)
  | 
0 | / aa b 
1 | | bdef
  | |  --- token (1:1-1:4, bytes 7-10)
  | |____^ parse (0:0-1:4, bytes 0-10)
2 |    aaa
  |    \\ cursor (2:1, byte 12)
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(Aa))
note: lexer state
 --> (0:0-2:4, bytes 0-15)
  | 
0 | / aa b 
1 | | bdef
2 | |  aaa
  | |  -- token (2:1-2:3, bytes 12-14)
  | |___^ parse (0:0-2:3, bytes 0-14)
  |      \\ cursor (2:3, byte 14)
");

    assert_eq!(lexer.next(), Some(A));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(A))
note: lexer state
 --> (0:0-2:4, bytes 0-15)
  | 
0 | / aa b 
1 | | bdef
2 | |  aaa
  | |    - token (2:3-2:4, bytes 14-15)
  | |____^ parse (0:0-2:4, bytes 0-15)
  |       \\ cursor (2:4, byte 15)
");

    assert_eq!(lexer.next(), None);
}



/// Tests `Lexer` display output formatting with tabstops.
#[test]
#[tracing::instrument]
fn tabstop_align() {
    use TestToken::*;
    let text = "\taa\ta\n\t\tb";
    let mut lexer = Lexer::new(Test::new(), text);

    assert_eq!(format!("{}", lexer), "\
Scanner: Test(None)
note: lexer state
 --> (0:0-0:9, bytes 0-5)
  | 
0 | 	aa	a
  | \\ token (0:0, byte 0)
  | \\ parse (0:0, byte 0)
  | \\ cursor (0:0, byte 0)
");

    assert_eq!(lexer.next(), Some(Ws));
    assert_eq!(format!("{}", lexer), "\
Scanner: Test(Some(Ws))
note: lexer state
 --> (0:0-0:9, bytes 0-5)
  | 
0 | 	aa	a
  | ---- token (0:0-0:4, bytes 0-1)
  | ---- parse (0:0-0:4, bytes 0-1)
  |     \\ cursor (0:4, byte 1)
");
}
