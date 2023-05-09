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
use crate::bracket_default_index;
use crate::delimited_list;
use crate::delimited_list_bounded;
use crate::sub;
use crate::unrecoverable;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;

// External library imports.
use ntest::timeout;
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::error::SourceError;
use tephra::Lexer;
use tephra::Pos;
use tephra::SourceText;
use tephra::Span;
use tephra::Spanned;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Test setup
////////////////////////////////////////////////////////////////////////////////
fn test_setup() {
    colored::control::set_override(false);
}

fn test_parser(text: &'static str) -> (
    Lexer<'static, Abc>,
    Context<'static, Abc>,
    Rc<RwLock<Vec<SourceError<&'static str>>>>,
    SourceText<&'static str>)
{
    let source = SourceText::new(text);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != AbcToken::Ws);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx_errors = errors.clone();
    let ctx = Context::new(Some(Box::new(move |e| 
        ctx_errors.write().unwrap().push(e.into_source_error(source))
    )));

    (lexer, ctx, errors, source)
}


////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn list_empty() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("");
    use AbcToken::*;

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(0, 0, 0));
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn list_one() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser(" abc ");
    use AbcToken::*;

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_one() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("[abc]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn list_two() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser(" abc,aac ");
    use AbcToken::*;

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_two() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("[abc,aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })),
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn list_one_failed() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("  ");
    use AbcToken::*;

    let actual = unrecoverable(
        sub(delimited_list_bounded(
            1, None,
            pattern,
            Comma, |_| false)))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: invalid item count
 --> (0:0-0:2, bytes 0-2)
  | 
0 |   
  |   \\ expected 1 item; found 0
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_zero() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("[]");
    use AbcToken::*;

    let (value, _succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .unwrap()
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_one_failed() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("[]");
    use AbcToken::*;

    let actual = unrecoverable(
        bracket_default_index(
            &[OpenBracket],
            delimited_list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: invalid item count
 --> (0:0-0:2, bytes 0-2)
  | 
0 | []
  |  \\ expected 1 item; found 0
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_one_recovered() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[       ]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: invalid item count
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [       ]
  |         \\ expected 1 item; found 0
");
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_two_recovered_first() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[   ,aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        None,
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [   ,aac]
  |     \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_two_recovered_second() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[abc,   ]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
            })),
            None,
        ],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [abc,   ]
  |         \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_missing_delimiter() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[abc aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![None], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: incomplete parse
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [abc aac]
  |     ^^^^^ unexpected text
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_nested() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("[[],[abc, abc], [aac]]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list(
                bracket_default_index(
                    &[OpenBracket],
                    delimited_list(
                        pattern,
                        Comma,
                        |tok| *tok == CloseBracket),
                    &[CloseBracket],
                    |_| false),
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            Some((vec![], 0)),
            Some((vec![
                Some(Pattern::Abc(Spanned {
                    value: "abc",
                    span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
                })),
                Some(Pattern::Abc(Spanned {
                    value: "abc",
                    span: Span::new_enclosing(Pos::new(10, 0, 10), Pos::new(13, 0, 13)),
                })),
            ], 0)),
            Some((vec![
                Some(Pattern::Xyc(Spanned {
                    value: "aac",
                    span: Span::new_enclosing(Pos::new(17, 0, 17), Pos::new(20, 0, 20)),
                })),
            ], 0)),
        ], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(22, 0, 22));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[timeout(100)]
fn bracket_list_commas() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[,,,,,abc]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            delimited_list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            None,
            None,
            None,
            None,
            None,
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::new_enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
            })),
        ], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));

    assert_eq!(errors.read().unwrap().len(), 5);
    assert_eq!(format!("{}", errors.write().unwrap().first().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [,,,,,abc]
  |  \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
    assert_eq!(format!("{}", errors.write().unwrap().last().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [,,,,,abc]
  |      \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
