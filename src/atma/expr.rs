////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma color expressions and selection types.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::span::Span;
use crate::result::Spanned;

// Standard library imports.
use std::borrow::Cow;

// External library imports.
use ::color::Color;


////////////////////////////////////////////////////////////////////////////////
// Expr types
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty,
    Color(Color),
    Reference(CellRef<'static>),
    Blend(BlendExpr),
}


#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
pub struct BlendExpr {
    pub blend_fn: BlendFunction,
    pub interpolate: Interpolate,
}


#[derive(Debug, Clone, PartialEq)]
pub enum BlendFunction {
    Unary(UnaryBlendFunction),
    Binary(BinaryBlendFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryBlendFunction {
    pub blend_method: UnaryBlendMethod,
    pub value: f32,
    pub arg: CellRef<'static>,
}

#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
pub struct BinaryBlendFunction {
    pub color_space: ColorSpace,
    pub blend_method: BinaryBlendMethod,
    pub arg_1: CellRef<'static>,
    pub arg_2: CellRef<'static>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    Rgb,
}



#[derive(Debug, Clone, PartialEq)]
pub struct Interpolate {
    pub color_space: ColorSpace,
    pub interpolate_fn: InterpolateFunction,
    pub amount: f32,
}



#[derive(Debug, Clone, PartialEq)]
pub struct InterpolateRange {
    pub color_space: ColorSpace,
    pub interpolate_fn: InterpolateFunction,
    pub start: f32,
    pub end: f32,
}

impl Default for InterpolateRange {
    fn default() -> Self {
        InterpolateRange {
            color_space: ColorSpace::Rgb,
            interpolate_fn: InterpolateFunction::Linear,
            start: 0.0,
            end: 1.0,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum InterpolateFunction {
    Linear,
    Cubic(f32, f32),
}

////////////////////////////////////////////////////////////////////////////////
// Selection types
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub enum CellRef<'name> {
    Index(u32),
    Position(Position),
    Name(Cow<'name, str>),
    Group {
        group: Cow<'name, str>,
        idx: u32,
    },
}

impl<'name> CellRef<'name> {
    /// Converts a `CellRef` to a static lifetime.
    pub fn into_static(self) -> CellRef<'static> {
        use CellRef::*;
        match self {
            Index(idx) => Index(idx),
            Position(position) => Position(position),
            Name(name) => Name(Cow::from(name.into_owned())),
            Group { group, idx } => Group {
                group: Cow::from(group.into_owned()),
                idx,
            },
        }
    }
}

impl<'name> From<PositionOrIndex> for CellRef<'name> {
    fn from(poi: PositionOrIndex) -> Self {
        match poi {
            PositionOrIndex::Position(pos) => CellRef::Position(pos),
            PositionOrIndex::Index(idx)    => CellRef::Index(idx),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub page: u16,
    pub line: u16,
    pub column: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionOrIndex {
    Index(u32),
    Position(Position),
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct CellSelection<'name>(pub Vec<CellSelector<'name>>);

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'text> {
    pub name: &'text str,
    pub args: Vec<Spanned<'text, FnArg>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnArg {
    U32(u32),
    F32(f32),
}

