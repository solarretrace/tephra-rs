////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::span::Span;


pub fn write_error_line<M>(
    f: &mut std::fmt::Formatter<'_>,
    msg: M)
    -> std::fmt::Result
    where M: std::fmt::Display,
{
    writeln!(f, "{} {}", "error:", msg)
}


pub fn write_source_info_line<M>(
    f: &mut std::fmt::Formatter<'_>,
    source_name: M,
    span: Span<'_>)
    -> std::fmt::Result
    where M: std::fmt::Display,
{
    writeln!(f, "  {} {}:{} (byte {})",
        "-->",
        source_name,
        span.page.start,
        span.byte.start)
}


pub fn write_gutter(
    f: &mut std::fmt::Formatter<'_>,
    width: usize,
    value: Option<usize>)
    -> std::fmt::Result
{
    if let Some(value) = value {
        write!(f, "{:<width$}|", value, width=width)
    } else {
        write!(f, "{:width$}|", "", width=width)
    }
}

