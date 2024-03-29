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
use crate::both;
use crate::one;
use crate::text;

// External library imports.
use ntest::timeout;
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::Lexer;
use tephra::Pos;
use tephra::Scanner;
use tephra::SourceText;
use tephra::SourceTextRef;

// Standard library imports.
use std::rc::Rc;


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
    fn new() -> Self {
        Self(None)
    }
}

impl Scanner for Test {
    type Token = TestToken;

    fn scan(&mut self, source: SourceTextRef<'_>, base: Pos)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source.as_ref()[base.byte..];
        let metrics = source.column_metrics();

        if text.starts_with("aa") {
            self.0 = Some(TestToken::Aa);
            Some((
                TestToken::Aa,
                metrics.end_position(&source.as_ref()[..base.byte + 2], base)))

        } else if text.starts_with('a') {
            self.0 = Some(TestToken::A);
            Some((
                TestToken::A,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with('b') {
            self.0 = Some(TestToken::B);
            Some((
                TestToken::B,
                metrics.end_position(&source.as_ref()[..=base.byte], base)))

        } else if text.starts_with("def") {
            self.0 = Some(TestToken::Def);
            Some((
                TestToken::Def,
                metrics.end_position(&source.as_ref()[..base.byte + 3], base)))
            
        } else {
            self.0 = Some(TestToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            if rest.len() < text.len() {
                let substr_len = text.len() - rest.len();
                let substr = &source.as_ref()[0.. base.byte + substr_len];
                Some((TestToken::Ws, metrics.end_position(substr, base)))
            } else {
                self.0 = None;
                None
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Test setup
////////////////////////////////////////////////////////////////////////////////
fn setup_test_environment() {
    colored::control::set_override(false);
}


////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Lexer::new`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::empty -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn empty() {
    setup_test_environment();

    const TEXT: &str = "";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);

    assert_eq!(
        lexer.next(),
        None);
}


/// Tests `Lexer::next`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::simple -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn simple() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(lexer.next(), Some(Ws));
    assert_eq!(lexer.next(), Some(B));
}


/// Tests `Lexer::peek`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::simple_peek -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn simple_peek() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);

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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::simple_iter -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn simple_iter() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);

    assert_eq!(
        lexer
            .iter_with_spans()
            .map(|lex| (
                lex.0,
                format!("{:?} ({})", source.clipped(lex.1).as_str(), lex.1)))
            .collect::<Vec<_>>(),
        vec![
            (Aa, "\"aa\" (0:0-0:2, bytes 0-2)".to_string()),
            (Ws, "\" \" (0:2-0:3, bytes 2-3)".to_string()),
            (B,  "\"b\" (0:3-0:4, bytes 3-4)".to_string()),
        ]);
}



/// Tests `Lexer` with whitespace filter.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::whitespace_filter -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn whitespace_filter() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b \nbdef\n aaa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);
    let _ = lexer.set_filter(Some(Rc::new(|tok| *tok != Ws)));

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (
            lex.0,
            format!("{:?} ({})", source.clipped(lex.1).as_str(), lex.1)))
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::both_whitespace_filter -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn both_whitespace_filter() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b \nbdef\n aaa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);
    let ctx = Context::empty();
    let _ = lexer.set_filter(Some(Rc::new(|tok| *tok != Ws)));

    let (actual, _) = both(
            text(one(Aa)),
            text(one(B)))
        (lexer, ctx)
        .unwrap()
        .take_value();

    let expected = ("aa", "b");

    assert_eq!(actual, expected);
}


/// Tests `Lexer` display output formatting.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::lexer_suite::display_formatting -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn display_formatting() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "aa b \nbdef\n aaa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);
    let _ = lexer.set_filter(Some(Rc::new(|tok| *tok != Ws)));

    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | \\ token (0:0, byte 0)
  | \\ parse (0:0, byte 0)
  | \\ cursor (0:0, byte 0), scanner: Test(None)
  | -- peek (0:0-0:2, bytes 0-2)
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | -- token (0:0-0:2, bytes 0-2)
  | -- parse (0:0-0:2, bytes 0-2)
  |   \\ cursor (0:2, byte 2), scanner: Test(Some(Aa))
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  |    - token (0:3-0:4, bytes 3-4)
  | ---- parse (0:0-0:4, bytes 0-4)
  |     \\ cursor (0:4, byte 4), scanner: Test(Some(B))
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-1:4, bytes 0-10)
  | 
0 | / aa b 
1 | | bdef
  | | - token (1:0-1:1, bytes 6-7)
  | |_^ parse (0:0-1:1, bytes 0-7)
  |    \\ cursor (1:1, byte 7), scanner: Test(Some(B))
");

    assert_eq!(lexer.next(), Some(Def));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-1:4, bytes 0-10)
  | 
0 | / aa b 
1 | | bdef
  | |  --- token (1:1-1:4, bytes 7-10)
  | |____^ parse (0:0-1:4, bytes 0-10)
  |       \\ cursor (1:4, byte 10), scanner: Test(Some(Def))
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-2:4, bytes 0-15)
  | 
0 | / aa b 
1 | | bdef
2 | |  aaa
  | |  -- token (2:1-2:3, bytes 12-14)
  | |___^ parse (0:0-2:3, bytes 0-14)
  |      \\ cursor (2:3, byte 14), scanner: Test(Some(Aa))
");

    assert_eq!(lexer.next(), Some(A));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-2:4, bytes 0-15)
  | 
0 | / aa b 
1 | | bdef
2 | |  aaa
  | |    - token (2:3-2:4, bytes 14-15)
  | |____^ parse (0:0-2:4, bytes 0-15)
  |       \\ cursor (2:4, byte 15), scanner: Test(Some(A))
");

    assert_eq!(lexer.next(), None);
}


/// Tests `Lexer` display output formatting with tabstops.
#[test]
#[timeout(50)]
fn tabstop_align() {
    setup_test_environment();

    use TestToken::*;
    const TEXT: &str = "\taa\ta\n\t\tb";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Test::new(), source);

    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-0:9, bytes 0-5)
  | 
0 | 	aa	a
  | \\ token (0:0, byte 0)
  | \\ parse (0:0, byte 0)
  | \\ cursor (0:0, byte 0), scanner: Test(None)
");

    assert_eq!(lexer.next(), Some(Ws));
    assert_eq!(format!("{}", lexer), "\
note: Lexer
 --> (0:0-0:9, bytes 0-5)
  | 
0 | 	aa	a
  | ---- token (0:0-0:4, bytes 0-1)
  | ---- parse (0:0-0:4, bytes 0-1)
  |     \\ cursor (0:4, byte 1), scanner: Test(Some(Ws))
");
}
