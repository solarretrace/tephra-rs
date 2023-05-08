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
use crate::bracket;
use crate::bracket_index;
use crate::one;
use crate::raw;
use crate::spanned;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;
use crate::unrecoverable;

// External library imports.
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::Lexer;
use tephra::Pos;
use tephra::SourceText;
use tephra::Span;
use tephra::Spanned;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test failed `bracket` combinator with error recovery, with missing brackets.
#[test]
fn recover_missing() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| 
        errors.write().unwrap().push(e.into_source_error(source))
    )));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected open bracket
 --> (0:0-0:5, bytes 0-5)
  | 
0 |  abc 
  |  \\ bracket expected here
");
}


/// Test failed `bracket` combinator with error recovery, with an unmatched open
/// bracket.
#[test]
fn recover_unmatched_open() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e|
            e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
#[test]
fn recover_unmatched_closed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched close bracket
 --> (0:0-0:5, bytes 0-5)
  | 
0 |  abc]
  |     ^ this bracket has no matching open
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
#[test]
fn recover_mismatched() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_index(
            &[OpenBracket, OpenBracket],
            pattern,
            &[CloseBracket, Comma], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: mismatched brackets
 --> (0:0-0:5, bytes 0-5)
  | 
0 | [abc,
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}

/// Test failed `bracket` combinator with error recovery, with an
/// unmatched bracket and raw error.
#[test]
fn recover_unmatched_raw() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched_raw");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = raw(bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched_raw:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator without error recovery, with an
/// unmatched bracket.
#[test]
fn recover_unmatched_unrecoverable() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched_unrecoverable");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = unrecoverable(bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched_unrecoverable:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test successful `bracket` combinator.
#[test]
fn comma_bracket_index() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "a,b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = spanned(bracket_index(
            &[A],
            one(Comma),
            &[B],
            |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = Spanned {
        value: (Some(Comma), 0),
        span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
    };
    println!("{:?}", actual);
    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Test successful `bracket` combinator.
#[test]
fn matching_both() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket_index(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                |_| false),
            bracket_index(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = ((Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 0), 
        (Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })), 0));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));
}

/// Test successful `bracket` combinator.
#[test]
fn matching_both_first_fail() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[a  ][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket], |_| false),
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = (
        None, 
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [a  ][aac]
  |  ^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}


/// Test `bracket` combinator failing due to mismatched brackets.
#[test]
fn matching_both_mismatch() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,[aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e.into_source_error(source)))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            bracket_index(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                |_| false),
            bracket_index(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: mismatched brackets
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [abc,[aac]
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}



