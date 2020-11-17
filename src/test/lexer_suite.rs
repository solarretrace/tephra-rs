////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::combinator::both;
use crate::combinator::one;
use crate::combinator::text;
use crate::combinator::repeat;
use crate::combinator::text_exact;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::position::Lf;
use crate::position::Pos;
use crate::result::ParseResultExt as _;


////////////////////////////////////////////////////////////////////////////////
// Token parser.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
struct Test;

impl Scanner for Test {
    type Token = TestToken;

    fn scan<'text, Cm>(&mut self, text: &'text str, metrics: Cm)
        -> Option<(Self::Token, Pos)>
        where Cm: ColumnMetrics,
    {
        // println!("{:?}", text);
        // println!("{:?}", text.split("\n").collect::<Vec<_>>());
        if text.starts_with("aa") {
            return Some((TestToken::Aa, metrics.width(&text[..2])));
        }
        if text.starts_with('a') {
            return Some((TestToken::A, metrics.width(&text[..1])));
        }
        if text.starts_with('b') {
            return Some((TestToken::B, metrics.width(&text[..1])));
        }
        if text.starts_with("def") {
            return Some((TestToken::Def, metrics.width(&text[..3])));
        }
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let substr = &text[0..substr_len];
            return Some((TestToken::Ws, metrics.width(substr)));
        }
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Lexer::new`.
#[test]
fn empty() {
    let text = "";
    let mut lexer = Lexer::new(Test, text, Lf::with_tab_width(4));

    assert_eq!(
        lexer.next(),
        None);
}


/// Tests `Lexer::next`.
#[test]
fn simple() {
    use TestToken::*;
    let text = "aa b";
    let mut lexer = Lexer::new(Test, text, Lf::with_tab_width(4));

    assert_eq!(
        lexer
            .iter_with_spans()
            .map(|lex| (lex.0, format!("{}", lex.1)))
            .collect::<Vec<_>>(),
        vec![
            (Aa, "\"aa\" (0:0-0:2, bytes 0-2)".to_string()),
            (Ws, "\" \" (0:2-0:3, bytes 2-3)".to_string()),
            (B,  "\"b\" (0:3-0:4, bytes 3-4)".to_string()),
        ]);
}


/// Tests `Lexer` with whitespace filter.
#[test]
fn no_whitespace() {
    use TestToken::*;
    let text = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (lex.0, format!("{}", lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
            (Aa,  "\"aa\" (0:0-0:2, bytes 0-2)".to_string()),
            (B,   "\"b\" (0:3-0:4, bytes 3-4)".to_string()),
            (B,   "\"b\" (1:0-1:1, bytes 6-7)".to_string()),
            (Def, "\"def\" (1:1-1:4, bytes 7-10)".to_string()),
            (Aa,  "\"aa\" (2:1-2:3, bytes 12-14)".to_string()),
            (A,   "\"a\" (2:3-2:4, bytes 14-15)".to_string()),
        ];

    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!();
    }

    assert_eq!(actual, expected);
}


/// Tests `both` with whitespace filter.
#[test]
fn atma_issue_1_both_no_whitespace() {
    use TestToken::*;
    let input = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test, input, Lf::with_tab_width(4));
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

/// Tests `both` with whitespace filter.
#[test]
fn atma_issue_1_both_no_whitespace_exact() {
    use TestToken::*;
    let input = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test, input, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            text_exact(one(Aa)),
            text_exact(one(B)))
        (lexer)
        .finish()
        .unwrap();

    let expected = ("aa", " b");

    assert_eq!(actual, expected);
}

/// Tests `repeat` with no successes.
#[test]
fn atma_issue_2_both_no_whitespace_exact() {
    use TestToken::*;
    let input = "aa aa aa";
    let mut lexer = Lexer::new(Test, input, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            repeat(0, None, one(B)),
            repeat(0, None, one(Aa)))
        (lexer)
        .finish()
        .unwrap();

    let expected = (0, 3);

    assert_eq!(actual, expected);
}


/// Tests `Lexer` display output formatting.
#[test]
fn display_formatting() {
    use TestToken::*;
    let text = "aa b \nbdef\n aaa";
    let mut lexer = Lexer::new(Test, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != Ws);

    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | ~ span
  | ~ full span
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | ~~ span
  | ~~ full span
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-0:5, bytes 0-5)
  | 
0 | aa b 
  | ~~~~ span
  | ~~~~ full span
");

    assert_eq!(lexer.next(), Some(B));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-1:4, bytes 0-10)
  | 
0 | // aa b 
1 | || bdef
  | ||_^ span
  |  |_^ full span
");

    assert_eq!(lexer.next(), Some(Def));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-1:4, bytes 0-10)
  | 
0 | // aa b 
1 | || bdef
  | ||____^ span
  |  |____^ full span
");

    assert_eq!(lexer.next(), Some(Aa));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-2:4, bytes 0-15)
  | 
0 | // aa b 
1 | || bdef
2 | ||  aaa
  | ||___^ span
  |  |___^ full span
");

    assert_eq!(lexer.next(), Some(A));
    assert_eq!(format!("{}", lexer), "\
note: lexer state
 --> (0:0-2:4, bytes 0-15)
  | 
0 | // aa b 
1 | || bdef
2 | ||  aaa
  | ||____^ span
  |  |____^ full span
");

    assert_eq!(lexer.next(), None);
}
