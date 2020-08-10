////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma expression and selection types.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::span::Span;

// Standard library imports.
use std::borrow::Cow;

// External library imports.
use ::color::Color;


////////////////////////////////////////////////////////////////////////////////
// Expr types
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Expr {
    Empty,
    Color(Color),
    Reference(CellRef<'static>),
    Blend(BlendExpr),
}


#[derive(Debug)]
pub enum InsertExpr {
    Ramp {
        count: u8,
        blend_fn: BlendFunction,
        interpolate: InterpolateRange,
    },
    Blend(BlendExpr),
    Color(Color),
    Copy(CellRef<'static>),
    Reference(CellRef<'static>),
}


#[derive(Debug)]
pub struct BlendExpr {
    pub blend_fn: BlendFunction,
    pub interpolate: Interpolate,
}


#[derive(Debug)]
pub enum BlendFunction {
    Unary(UnaryBlendFunction),
    Binary(BinaryBlendFunction),
}

#[derive(Debug)]
pub struct UnaryBlendFunction {
    pub blend_method: UnaryBlendMethod,
    pub value: f32,
    pub arg: CellRef<'static>,
}

#[derive(Debug)]
pub enum UnaryBlendMethod {
    SetRed,
    SetGreen,
    SetBlue,
    HueShift,
    SetHue,
    Saturate,
    Desaturate,
    Lighten,
    Darken,
}


#[derive(Debug)]
pub struct BinaryBlendFunction {
    pub color_space: ColorSpace,
    pub blend_method: BinaryBlendMethod,
    pub arg_1: CellRef<'static>,
    pub arg_2: CellRef<'static>,
}

#[derive(Debug)]
pub enum BinaryBlendMethod {
    Blend,
    Multiply,
    Divide,
    Subtract,
    Difference,
    Screen,
    Overlay,
    HardLight,
    SoftLight,
    ColorDodge,
    ColorBurn,
    VividLight,
    LinearDodge,
    LinearBurn,
    LinearLight,
}

#[derive(Debug)]
pub enum ColorSpace {
    Rgb,
}



#[derive(Debug)]
pub struct Interpolate {
    pub color_space: ColorSpace,
    pub interpolate_fn: InterpolateFunction,
    pub amount: f32,
}



#[derive(Debug)]
pub struct InterpolateRange {
    pub color_space: ColorSpace,
    pub interpolate_fn: InterpolateFunction,
    pub start: f32,
    pub end: f32,
}


#[derive(Debug)]
pub enum InterpolateFunction {
    Linear,
    Cubic(f32, f32),
}

////////////////////////////////////////////////////////////////////////////////
// Selection types
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
pub enum CellRef<'name> {
    Index(u32),
    Position(Position),
    Name(Cow<'name, str>),
    Group {
        group: Cow<'name, str>,
        idx: u32,
    },
}

impl<'name> From<PositionOrIndex> for CellRef<'name> {
    fn from(poi: PositionOrIndex) -> Self {
        match poi {
            PositionOrIndex::Position(pos) => CellRef::Position(pos),
            PositionOrIndex::Index(idx)    => CellRef::Index(idx),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub page: u16,
    pub line: u16,
    pub column: u16,
}

#[derive(Debug, PartialEq)]
pub enum PositionOrIndex {
    Index(u32),
    Position(Position),
}

#[derive(Debug, PartialEq)]
pub struct PositionSelector {
    pub page: Option<u16>,
    pub line: Option<u16>,
    pub column: Option<u16>,
}

impl From<Position> for PositionSelector {
    fn from(pos: Position) -> Self {
        PositionSelector {
            page: Some(pos.page),
            line: Some(pos.line),
            column: Some(pos.column),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CellSelection<'name>(pub Vec<CellSelector<'name>>);

#[derive(Debug, PartialEq)]
pub enum CellSelector<'name> {
    All,
    Index(u32),
    IndexRange {
        low: u32,
        high: u32,
    },
    PositionSelector(PositionSelector),
    PositionRange {
        low: Position,
        high: Position
    },
    Name(Cow<'name, str>),
    Group {
        group: Cow<'name, str>,
        idx: u32,
    },
    GroupRange {
        group: Cow<'name, str>,
        low: u32,
        high: u32,
    },
    GroupAll(Cow<'name, str>),
}


////////////////////////////////////////////////////////////////////////////////
// Function calls
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct FnCall<'text, Cm> {
    pub name: &'text str,
    pub args: Vec<(FnArg, Span<'text>)>,
}

#[derive(Debug)]
pub enum FnArg {
    U32(u32),
    F32(f32),
}
