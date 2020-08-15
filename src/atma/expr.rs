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
use std::str::FromStr;

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
    Ramp(RampExpr),
    Blend(BlendExpr),
    Color(Color),
    Copy(CellRef<'static>),
    Reference(CellRef<'static>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RampExpr {
    pub count: u8,
    pub blend_fn: BlendFunction,
    pub interpolate: InterpolateRange,
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

impl FromStr for UnaryBlendMethod {
    type Err = InvalidBlendMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UnaryBlendMethod::*;
        match s {
            "set_red"    => Ok(SetRed),
            "set_green"  => Ok(SetGreen),
            "set_blue"   => Ok(SetBlue),
            "hue_shift"  => Ok(HueShift),
            "set_hue"    => Ok(SetHue),
            "saturate"   => Ok(Saturate),
            "desaturate" => Ok(Desaturate),
            "lighten"    => Ok(Lighten),
            "darken"     => Ok(Darken),
            _            => Err(InvalidBlendMethod)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryBlendFunction {
    pub blend_method: BinaryBlendMethod,
    pub color_space: ColorSpace,
    pub arg_0: CellRef<'static>,
    pub arg_1: CellRef<'static>,
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

impl FromStr for BinaryBlendMethod {
    type Err = InvalidBlendMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BinaryBlendMethod::*;
        match s {
            "blend"        => Ok(Blend),
            "multiply"     => Ok(Multiply),
            "divide"       => Ok(Divide),
            "subtract"     => Ok(Subtract),
            "difference"   => Ok(Difference),
            "screen"       => Ok(Screen),
            "overlay"      => Ok(Overlay),
            "hard_light"   => Ok(HardLight),
            "soft_light"   => Ok(SoftLight),
            "color_dodge"  => Ok(ColorDodge),
            "color_burn"   => Ok(ColorBurn),
            "vivid_light"  => Ok(VividLight),
            "linear_dodge" => Ok(LinearDodge),
            "linear_burn"  => Ok(LinearBurn),
            "linear_light" => Ok(LinearLight),
            _              => Err(InvalidBlendMethod)
        }
    }
}

pub struct InvalidBlendMethod;

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

impl Default for Interpolate {
    fn default() -> Self {
        Interpolate {
            color_space: ColorSpace::Rgb,
            interpolate_fn: InterpolateFunction::Linear,
            amount: 0.0,
        }
    }
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

