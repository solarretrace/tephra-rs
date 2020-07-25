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
use crate::span::NewLine;



pub fn write_error_line<M>(
    f: &mut std::fmt::Formatter<'_>,
    msg: M)
    -> std::fmt::Result
    where M: std::fmt::Display,
{
    writeln!(f, "{} {}", "error:", msg)
}



pub fn write_source_span<M, Nl>(
    f: &mut std::fmt::Formatter<'_>,
    source_name: M,
    span: Span<'_, Nl>)
    -> std::fmt::Result
    where
        M: std::fmt::Display,
        Nl: NewLine,
{
    let gutter_width: usize = std::cmp::max(
        (span.page.end.line as f32).log10().ceil() as usize, 1);

    write_source_info_line(f, gutter_width, source_name, span)?;
    write_source_line(f, gutter_width, None, "")?;
    for line_span in span.split_lines() {
        write_source_line(
            f,
            gutter_width,
            Some(line_span.page.start.line),
            span.widen_to_line().text())?;
    }
    write_source_line(f, gutter_width, None, "")
    // for line in 
}

pub fn write_source_info_line<M, Nl>(
    f: &mut std::fmt::Formatter<'_>,
    gutter_width: usize,
    source_name: M,
    span: Span<'_, Nl>)
    -> std::fmt::Result
    where M: std::fmt::Display
{
    writeln!(f, "{:width$}{} {}:{} (byte {})",
        "",
        "-->",
        source_name,
        span.page.start,
        span.byte.start,
        width=gutter_width)
}

pub fn write_source_line<M>(
    f: &mut std::fmt::Formatter<'_>,
    gutter_width: usize,
    line_number: Option<usize>,
    msg: M)
    -> std::fmt::Result
    where M: std::fmt::Display
{
    if let Some(line_num) = line_number {
        writeln!(f, "{:<width$} | {}", line_num, msg, width=gutter_width)
    } else {
        writeln!(f, "{:width$} | {}", "", msg, width=gutter_width)
    }
}

