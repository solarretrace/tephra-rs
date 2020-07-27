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
use crate::span::Lf;
use crate::lexer::Lexer;
use crate::test::atma_expr::*;

////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Lexer::new` for the AtmaExprScanner.
#[test]
fn empty() {
    let text = "";
    let as_tok = AtmaExprScanner::new();
    let mut lexer = Lexer::new(as_tok, text, Lf);

    let actual = lexer.next();
    let expected = None;
    assert_eq!(actual, expected);
}

////////////////////////////////////////////////////////////////////////////////
// String tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests AtmaExprScanner with an empty single-quoted string.
#[test]
fn string_single_empty() {
    use AtmaToken::*;
    let text = "''";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringCloseSingle, "\"'\" (0:1-0:2, bytes 1-2)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with an unclosed single-quoted string.
#[test]
#[should_panic]
fn string_single_unclosed() {
    use AtmaToken::*;
    let text = "'abc";
    let as_tok = AtmaExprScanner::new();
    let mut lexer = Lexer::new(as_tok, text, Lf)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaExprScanner with a non-empty single-quoted string.
#[test]
fn string_single_text() {
    use AtmaToken::*;
    let text = "'abc \n xyz'";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
        (StringCloseSingle, "\"'\" (1:4-1:5, bytes 10-11)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with a quote-containing single-quoted string.
#[test]
fn string_single_quotes() {
    use AtmaToken::*;
    let text = "'abc\"\n\\'xyz'";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc\"\n\\'xyz\" (0:1-1:5, bytes 1-11)".to_owned()),
        (StringCloseSingle, "\"'\" (1:5-1:6, bytes 11-12)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaExprScanner with an empty double-quoted string.
#[test]
fn string_double_empty() {
    use AtmaToken::*;
    let text = "\"\"";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringCloseDouble, "\"\"\" (0:1-0:2, bytes 1-2)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaExprScanner with an unclosed double-quoted string.
#[test]
#[should_panic]
fn string_double_unclosed() {
    use AtmaToken::*;
    let text = "\"abc";
    let as_tok = AtmaExprScanner::new();
    let mut lexer = Lexer::new(as_tok, text, Lf)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}


/// Tests AtmaExprScanner with a non-empty double-quoted string.
#[test]
fn string_double_text() {
    use AtmaToken::*;
    let text = "\"abc \n xyz\"";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
        (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 10-11)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with a quote-containing double-quoted string.
#[test]
fn string_double_quotes() {
    use AtmaToken::*;
    let text = "\"abc\\\"\n'xyz\"";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc\\\"\n'xyz\" (0:1-1:4, bytes 1-11)".to_owned()),
        (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 11-12)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaExprScanner with an empty raw-quoted string.
#[test]
fn string_raw_empty() {
    use AtmaToken::*;
    let text = "r\"\"";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r\"\" (0:0-0:2, bytes 0-2)".to_owned()),
        (RawStringClose, "\"\"\" (0:2-0:3, bytes 2-3)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaExprScanner with an empty raw-quoted string using hashes.
#[test]
fn string_raw_empty_hashed() {
    use AtmaToken::*;
    let text = "r##\"\"##";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r##\"\" (0:0-0:4, bytes 0-4)".to_owned()),
        (RawStringClose, "\"\"##\" (0:4-0:7, bytes 4-7)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with an unclosed raw-quoted string.
#[test]
#[should_panic]
fn string_raw_unclosed() {
    use AtmaToken::*;
    let text = "r###\"abc";
    let as_tok = AtmaExprScanner::new();
    let mut lexer = Lexer::new(as_tok, text, Lf)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaExprScanner with an mismatched raw-quoted string.
#[test]
#[should_panic]
fn string_raw_mismatched() {
    use AtmaToken::*;
    let text = "r###\"abc\"#";
    let as_tok = AtmaExprScanner::new();
    let mut lexer = Lexer::new(as_tok, text, Lf)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaExprScanner with a non-empty raw-quoted string.
#[test]
fn string_raw_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r########\"\" (0:0-0:10, bytes 0-10)".to_owned()),
        (RawStringText,  "\"abc \n xyz\" (0:10-1:4, bytes 10-19)".to_owned()),
        (RawStringClose, "\"\"########\" (1:4-1:13, bytes 19-28)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with a non-empty raw-quoted string with quotes
/// inside.
#[test]
fn string_raw_quoted_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaExprScanner::new();
    let lexer = Lexer::new(as_tok, text, Lf);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r########\"\" (0:0-0:10, bytes 0-10)".to_owned()),
        (RawStringText,  "\"abc \n xyz\" (0:10-1:4, bytes 10-19)".to_owned()),
        (RawStringClose, "\"\"########\" (1:4-1:13, bytes 19-28)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}
