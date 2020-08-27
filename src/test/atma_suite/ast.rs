////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma AST tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::lexer::Lexer;
use crate::combinator::end_of_text;
use crate::combinator::left;
use crate::position::Lf;
use crate::position::Pos;
use crate::result::Spanned;
use crate::result::ParseResultExt as _;
use crate::span::Span;

// External crate imports.
use ::color::Color;
use ::color::Rgb;


////////////////////////////////////////////////////////////////////////////////
// PrimaryExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `primary_expr` with an Ident value.
#[test]
fn primary_expr_ident() {
    let text = "abcd";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Ident("abcd"),
        "\"abcd\" (0:0-0:4, bytes 0-4)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `primary_expr` with an Float value.
#[test]
fn primary_expr_float() {
    let text = "123.45";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Float("123.45"),
        "\"123.45\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an Uint value.
#[test]
fn primary_expr_uint() {
    let text = "12345";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Uint("12345"),
        "\"12345\" (0:0-0:5, bytes 0-5)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `primary_expr` with an Uint value with parens.
#[test]
fn primary_expr_uint_parens() {
    let text = "(12345)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Tuple(vec![
            AstExpr::Unary(Spanned {
                value: UnaryExpr::Call(
                    CallExpr::Primary(
                        PrimaryExpr::Uint("12345"))),
                span: Span::new_enclosing(
                    Pos::new(1, 0, 1),
                    Pos::new(6, 0, 6),
                    text),
            }),
        ]),
        "\"(12345)\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `primary_expr` with an Uint value with brackets.
#[test]
fn primary_expr_uint_brackets() {
    let text = "[12345]";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Array(vec![
            AstExpr::Unary(Spanned {
                value: UnaryExpr::Call(
                    CallExpr::Primary(
                        PrimaryExpr::Uint("12345"))),
                span: Span::new_enclosing(
                    Pos::new(1, 0, 1),
                    Pos::new(6, 0, 6),
                    text),
            }),
        ]),
        "\"[12345]\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an Color value.
#[test]
fn primary_expr_color() {
    let text = "#ABCDEF";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Color(Color::from(Rgb::from(0xABCDEF))),
        "\"#ABCDEF\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an CellRef value.
#[test]
fn primary_expr_cell_ref_index() {
    let text = ":0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::CellRef(CellRef::Index(0)),
        "\":0\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an CellRef value.
#[test]
fn primary_expr_cell_ref_name() {
    let text = "'abc'";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::CellRef(CellRef::Name("abc".into())),
        "\"'abc'\" (0:0-0:5, bytes 0-5)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `cell_ref` with an index value overflow.
#[test]
fn primary_expr_cell_ref_invalid() {
    let text = ":0A";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in lexer.clone() {
    //     println!("{:?}", tok);
    // }

    let failure = left(
            cell_ref,
            end_of_text)
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "expected end of text",
        "\":0\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

// ////////////////////////////////////////////////////////////////////////////////
// // CallExpr
// ////////////////////////////////////////////////////////////////////////////////

/// Tests `call_expr` with an Call value.
#[test]
fn call_expr_call() {
    let text = "abcd( a, #12AB99, 12.34 )";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = call_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CallExpr::Call {
            operand: Box::new(Spanned {
                value: CallExpr::Primary(PrimaryExpr::Ident("abcd")),
                span: Span::new_enclosing(
                    Pos::new(0, 0, 0),
                    Pos::new(4, 0, 4),
                    text),
            }),
            args: vec![
                AstExpr::Unary(Spanned {
                    value: UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Ident("a"))),
                    span: Span::new_enclosing(
                        Pos::new(6, 0, 6),
                        Pos::new(7, 0, 7),
                        text),
                }),
                AstExpr::Unary(Spanned {
                    value: UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Color(
                                Color::from(Rgb::from(0x12AB99))))),
                    span: Span::new_enclosing(
                        Pos::new(9, 0, 9),
                        Pos::new(16, 0, 16),
                        text),
                }),
                AstExpr::Unary(Spanned {
                    value: UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Float("12.34"))),
                    span: Span::new_enclosing(
                        Pos::new(18, 0, 18),
                        Pos::new(23, 0, 23),
                        text),
                }),
            ],
        },
        "\"( a, #12AB99, 12.34 )\" (0:4-0:25, bytes 4-25)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}



////////////////////////////////////////////////////////////////////////////////
// AstExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `call_expr` with an Call value.
#[test]
fn ast_expr_call_negate() {
    let text = "-(abcd)( \na, \n#12AB99, \n12.34 )";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = ast_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        AstExpr::Unary(Spanned {
            value: UnaryExpr::Minus {
                op: Span::new_enclosing(
                    Pos::new(0, 0, 0),
                    Pos::new(1, 0, 1),
                    text),
                operand: Box::new(Spanned {
                    value: UnaryExpr::Call(
                        CallExpr::Call {
                            operand: Box::new(Spanned {
                                value: CallExpr::Primary(
                                    PrimaryExpr::Tuple(vec![
                                        AstExpr::Unary(Spanned {
                                            value: UnaryExpr::Call(
                                                CallExpr::Primary(
                                                    PrimaryExpr::Ident("abcd"))),
                                            span: Span::new_enclosing(
                                                Pos::new(2, 0, 2),
                                                Pos::new(6, 0, 6),
                                                text),
                                        }),
                                    ])
                                ),
                                span: Span::new_enclosing(
                                    Pos::new(1, 0, 1),
                                    Pos::new(7, 0, 7),
                                    text),
                            }),
                            args: vec![
                                AstExpr::Unary(Spanned {
                                    value: UnaryExpr::Call(
                                        CallExpr::Primary(
                                            PrimaryExpr::Ident("a"))),
                                    span: Span::new_enclosing(
                                        Pos::new(10, 1, 0),
                                        Pos::new(11, 1, 1),
                                        text),
                                }),
                                AstExpr::Unary(Spanned {
                                    value: UnaryExpr::Call(
                                        CallExpr::Primary(
                                            PrimaryExpr::Color(
                                                Color::from(Rgb::from(0x12AB99))))),
                                    span: Span::new_enclosing(
                                        Pos::new(14, 2, 0),
                                        Pos::new(21, 2, 7),
                                        text),
                                }),
                                AstExpr::Unary(Spanned {
                                    value: UnaryExpr::Call(
                                        CallExpr::Primary(
                                            PrimaryExpr::Float("12.34"))),
                                    span: Span::new_enclosing(
                                        Pos::new(24, 3, 0),
                                        Pos::new(29, 3, 5),
                                        text),
                                }),
                            ],
                        }),
                    span: Span::new_enclosing(
                        Pos::new(1, 0, 1),
                        Pos::new(31, 3, 7),
                        text),
                }),
            },
            span: Span::new_enclosing(
                Pos::new(0, 0, 0),
                Pos::new(31, 3, 7),
                text),
        }),
        "\"-(abcd)( \na, \n#12AB99, \n12.34 )\" (0:0-3:7, bytes 0-31)"
            .to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

////////////////////////////////////////////////////////////////////////////////
// Ast Color match
////////////////////////////////////////////////////////////////////////////////

/// Tests `ast_expr` with an Color value produced by matching.
#[test]
fn ast_expr_cell_ref() {
    let text = "rgb(0.3, 0.6, 0.9)";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = Color::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = Color::from(Rgb::from([0.3, 0.6, 0.9]));

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// Ast InterpolateRange match
////////////////////////////////////////////////////////////////////////////////

/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_linear() {
    let text = "linear";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Linear,
        start: 0.0,
        end: 1.0,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_cubic() {
    let text = "cubic";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
        start: 0.0,
        end: 1.0,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_cubic_parameterized() {
    let text = "cubic(0.3, 4.5)";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Cubic(0.3, 4.5),
        start: 0.0,
        end: 1.0,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_cubic_parameterized_ranged() {
    let text = "cubic(0.3, 4.5)([0.2, 0.8])";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Cubic(0.3, 4.5),
        start: 0.2,
        end: 0.8,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_cubic_ranged() {
    let text = "cubic([0.2, 0.8])";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
        start: 0.2,
        end: 0.8,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `ast_expr` with an InterpolateRange value produced by matching.
#[test]
fn ast_interpolate_range_cubic_ranged_rgb() {
    let text = "cubic([0.2, 0.8], rgb)";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = InterpolateRange::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = InterpolateRange {
        color_space: ColorSpace::Rgb,
        interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
        start: 0.2,
        end: 0.8,
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

////////////////////////////////////////////////////////////////////////////////
// Ast BlendExpr match
////////////////////////////////////////////////////////////////////////////////

/// Tests `ast_expr` with an BlendExpr value produced by matching.
#[test]
fn ast_blend_expr_set_red() {
    let text = "set_red(:0, 2.3)";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in lexer.clone() {
    //     println!("{:?}", tok);
    // }

    let actual = BlendExpr::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = BlendExpr {
        blend_fn: BlendFunction::Unary(UnaryBlendFunction {
            blend_method: UnaryBlendMethod::SetRed,
            value: 2.3,
            arg: CellRef::Index(0),
        }),
        interpolate: Interpolate {
            color_space: ColorSpace::Rgb,
            interpolate_fn: InterpolateFunction::Linear,
            amount: 0.0,
        },
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `ast_expr` with an BlendExpr value produced by matching.
#[test]
fn ast_blend_expr_desaturate() {
    let text = "desaturate('abc', 2.3,\n cubic(1.0, 2.0)(0.5, rgb))";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in lexer.clone() {
    //     println!("{:?}", tok);
    // }

    let actual = BlendExpr::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = BlendExpr {
        blend_fn: BlendFunction::Unary(UnaryBlendFunction {
            blend_method: UnaryBlendMethod::Desaturate,
            value: 2.3,
            arg: CellRef::Name("abc".into()),
        }),
        interpolate: Interpolate {
            color_space: ColorSpace::Rgb,
            interpolate_fn: InterpolateFunction::Cubic(1.0, 2.0),
            amount: 0.5,
        },
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

////////////////////////////////////////////////////////////////////////////////
// Ast RampExpr match
////////////////////////////////////////////////////////////////////////////////

/// Tests `ast_expr` with an BlendExpr value produced by matching.
#[test]
fn ast_ramp_expr() {
    let text = "ramp(3, soft_light(:0, :3))";
    let scanner = AtmaScanner::new();
    let metrics = Lf::with_tab_width(4);
    let mut lexer = Lexer::new(scanner, text, metrics);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in lexer.clone() {
    //     println!("{:?}", tok);
    // }

    let actual = RampExpr::match_expr(
        ast_expr
            (lexer)
            .finish()
            .unwrap(),
        metrics)
        .unwrap();

    let expected = RampExpr {
        count: 3,
        blend_fn: BlendFunction::Binary(BinaryBlendFunction {
            blend_method: BinaryBlendMethod::SoftLight,
            color_space: ColorSpace::default(),
            arg_0: CellRef::Index(0),
            arg_1: CellRef::Index(3),
        }),
        interpolate: InterpolateRange::default(),
    };

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

