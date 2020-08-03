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
use crate::lexer::Scanner;
use crate::lexer::Lexer;
use crate::span::Pos;
use crate::span::Lf;
use crate::span::NewLine;


////////////////////////////////////////////////////////////////////////////////
// Token parser.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenError(&'static str);
impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestToken {
    Aa,
    A,
    B,
    Def,
    Ws,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Test;

impl Scanner for Test {
    type Token = TestToken;

    fn scan<'text, Nl>(&mut self, text: &'text str)
        -> Option<(Self::Token, Pos)>
        where Nl: NewLine,
    {
        // println!("{:?}", text);
        // println!("{:?}", text.split("\n").collect::<Vec<_>>());
        if text.starts_with("aa") {
            return Some((TestToken::Aa, Pos::new(2, 0, 2)));
        }
        if text.starts_with('a') {
            return Some((TestToken::A, Pos::new(1, 0, 1)));
        }
        if text.starts_with('b') {
            return Some((TestToken::B, Pos::new(1, 0, 1)));
        }
        if text.starts_with("def") {
            return Some((TestToken::Def, Pos::new(3, 0, 3)));
        }
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let substr = &text[0..substr_len];
            let span = Pos::new_from_string::<_, Nl>(substr);
            return Some((TestToken::Ws, span));
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
    let mut lexer = Lexer::new(Test, text, Lf);

    assert_eq!(
        lexer.next(),
        None);
}


/// Tests `Lexer::next`.
#[test]
fn simple() {
    use TestToken::*;
    let text = "aa b";
    let mut lexer = Lexer::new(Test, text, Lf);

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
    let mut lexer = Lexer::new(Test, text, Lf);
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
