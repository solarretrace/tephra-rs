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
use crate::lexer::Lexer;
use crate::test::atma_script::*;



////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////


/// Tests `Lexer::new` for the AtmaScriptScanner.
#[test]
fn as_lexer_empty() {
    let text = "";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.next(),
        None);
}

/// Tests AtmaScriptScanner with a line comment.
#[test]
fn as_lexer_line_comment() {
    use AtmaToken::*;
    let text = "#abc\n";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (LineCommentOpen, "\"#\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentText, "\"abc\" (0:1-0:4, bytes 1-4)".to_owned()),
        (Whitespace,      "\"\n\" (0:4-1:0, bytes 4-5)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScanner with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comment_circumfix_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t\n ";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (Whitespace,      "\"\n\t \n\" (0:0-2:0, bytes 0-4)".to_owned()),
        (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
        (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
        (Whitespace,      "\"\n \t\n \" (2:4-4:1, bytes 8-13)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScanner with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comments_remove_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t#def\n ";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok| *tok != Whitespace);

    assert_eq!(
        lexer
            .map(|res| {
                let lex = res.unwrap();
                (*lex.token(), format!("{}", lex.span()))
            })
            .collect::<Vec<_>>(),
        vec![
            (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
            (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
            (LineCommentOpen, "\"#\" (3:2-3:3, bytes 11-12)".to_owned()),
            (LineCommentText, "\"def\" (3:3-3:6, bytes 12-15)".to_owned()),
        ]);
}

/// Tests AtmaScriptScanner with an empty single-quoted string.
#[test]
fn as_lexer_string_single_empty() {
    use AtmaToken::*;
    let text = "''";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with an unclosed single-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_single_unclosed() {
    use AtmaToken::*;
    let text = "'abc";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text)
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

/// Tests AtmaScriptScanner with a non-empty single-quoted string.
#[test]
fn as_lexer_string_single_text() {
    use AtmaToken::*;
    let text = "'abc \n xyz'";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with a quote-containing single-quoted string.
#[test]
fn as_lexer_string_single_quotes() {
    use AtmaToken::*;
    let text = "'abc\"\n\\'xyz'";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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


/// Tests AtmaScriptScanner with an empty double-quoted string.
#[test]
fn as_lexer_string_double_empty() {
    use AtmaToken::*;
    let text = "\"\"";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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


/// Tests AtmaScriptScanner with an unclosed double-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_double_unclosed() {
    use AtmaToken::*;
    let text = "\"abc";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text)
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


/// Tests AtmaScriptScanner with a non-empty double-quoted string.
#[test]
fn as_lexer_string_double_text() {
    use AtmaToken::*;
    let text = "\"abc \n xyz\"";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with a quote-containing double-quoted string.
#[test]
fn as_lexer_string_double_quotes() {
    use AtmaToken::*;
    let text = "\"abc\\\"\n'xyz\"";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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


/// Tests AtmaScriptScanner with an empty raw-quoted string.
#[test]
fn as_lexer_string_raw_empty() {
    use AtmaToken::*;
    let text = "r\"\"";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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


/// Tests AtmaScriptScanner with an empty raw-quoted string using hashes.
#[test]
fn as_lexer_string_raw_empty_hashed() {
    use AtmaToken::*;
    let text = "r##\"\"##";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with an unclosed raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_raw_unclosed() {
    use AtmaToken::*;
    let text = "r###\"abc";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text)
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

/// Tests AtmaScriptScanner with an mismatched raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_raw_mismatched() {
    use AtmaToken::*;
    let text = "r###\"abc\"#";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text)
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

/// Tests AtmaScriptScanner with a non-empty raw-quoted string.
#[test]
fn as_lexer_string_raw_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with a non-empty raw-quoted string with quotes
/// inside.
#[test]
fn as_lexer_string_raw_quoted_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


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

/// Tests AtmaScriptScanner with a CommandChunk.
#[test]
fn as_lexer_command_chunk() {
    use AtmaToken::*;
    let text = "abc-def";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandChunk,  "\"abc-def\" (0:0-0:7, bytes 0-7)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScanner with a combination of tokens.
#[test]
fn as_lexer_combined() {
    use AtmaToken::*;
    let text = "# \n\n \"abc\\\"\"'def' r##\"\t\"##\n\n\n--zyx--wvut";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (LineCommentOpen,   "\"#\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentText,   "\" \" (0:1-0:2, bytes 1-2)".to_owned()),
        (Whitespace,        "\"\n\n \" (0:2-2:1, bytes 2-5)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 5-6)".to_owned()),
        (StringText,        "\"abc\\\"\" (2:2-2:7, bytes 6-11)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:7-2:8, bytes 11-12)".to_owned()),
        (StringOpenSingle,  "\"'\" (2:8-2:9, bytes 12-13)".to_owned()),
        (StringText,        "\"def\" (2:9-2:12, bytes 13-16)".to_owned()),
        (StringCloseSingle, "\"'\" (2:12-2:13, bytes 16-17)".to_owned()),
        (Whitespace,        "\" \" (2:13-2:14, bytes 17-18)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:14-2:18, bytes 18-22)".to_owned()),
        (RawStringText,     "\"\t\" (2:18-2:19, bytes 22-23)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:19-2:22, bytes 23-26)".to_owned()),
        (Whitespace,        "\"\n\n\n\" (2:22-5:0, bytes 26-29)".to_owned()),
        (CommandChunk,      "\"--zyx--wvut\" (5:0-5:11, bytes 29-40)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScanner with a combination of tokens separated by
/// terminators.
#[test]
fn as_lexer_combined_terminated() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandTerminator, "\";\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentOpen,   "\"#\" (0:1-0:2, bytes 1-2)".to_owned()),
        (LineCommentText,   "\"; \" (0:2-0:4, bytes 2-4)".to_owned()),
        (Whitespace,        "\"\n\n \" (0:4-2:1, bytes 4-7)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (CommandTerminator, "\";\" (2:9-2:10, bytes 15-16)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (Whitespace,        "\" \" (2:16-2:17, bytes 22-23)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandTerminator, "\";\" (2:26-2:27, bytes 32-33)".to_owned()),
        (Whitespace,        "\"\n\n\" (2:27-4:0, bytes 33-35)".to_owned()),
        (CommandTerminator, "\";\" (4:0-4:1, bytes 35-36)".to_owned()),
        (Whitespace,        "\"\n\" (4:1-5:0, bytes 36-37)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandTerminator, "\";\" (5:5-5:6, bytes 42-43)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScanner with a combination of tokens separated by
/// terminators with whitespace filtered out.
#[test]
fn as_lexer_combined_terminated_remove_whitespace() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok| *tok != Whitespace);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandTerminator, "\";\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentOpen,   "\"#\" (0:1-0:2, bytes 1-2)".to_owned()),
        (LineCommentText,   "\"; \" (0:2-0:4, bytes 2-4)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (CommandTerminator, "\";\" (2:9-2:10, bytes 15-16)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandTerminator, "\";\" (2:26-2:27, bytes 32-33)".to_owned()),
        (CommandTerminator, "\";\" (4:0-4:1, bytes 35-36)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandTerminator, "\";\" (5:5-5:6, bytes 42-43)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScanner with a combination of tokens separated by
/// terminators with multiple tokens filtered out.
#[test]
fn as_lexer_combined_terminated_filtered() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScanner::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok|
        *tok != Whitespace &&
        *tok != LineCommentOpen &&
        *tok != LineCommentText &&
        *tok != CommandTerminator);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}
