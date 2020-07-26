////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error display helper functions.
////////////////////////////////////////////////////////////////////////////////
    // TODO: This module is currently under development.
    #![allow(unused)]
    #![allow(missing_docs)]

// Local imports.
use crate::span::Span;
use crate::span::SplitLines;
use crate::span::NewLine;





////////////////////////////////////////////////////////////////////////////////
// SourceSpan
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct SourceSpan<'text, 'msg, Nl> {
    span: Span<'text, Nl>,
    message_type: (),
    message: &'msg str,
    source_name: &'msg str,
    highlights: Vec<Highlight<'text, 'msg, Nl>>,
}


impl<'text, 'msg, Nl> SourceSpan<'text, 'msg, Nl> {
    pub fn new(span: Span<'text, Nl>, message: &'msg str) -> Self {
        SourceSpan {
            span,
            message_type: (),
            message,
            source_name: "",
            highlights: Vec::with_capacity(1),
        }
    }

    pub fn with_source_name(mut self, source_name: &'msg str) -> Self {
        self.source_name = source_name;
        self
    }

    pub fn with_highlight(mut self, highlight: Highlight<'text, 'msg, Nl>)
        -> Self
    {
        self.span = self.span.enclose(&highlight.span);
        self.highlights.push(highlight);
        self
    }
}

impl<'text, 'msg, Nl> std::fmt::Display for SourceSpan<'text, 'msg, Nl>
    where Nl: NewLine
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", "error:", self.message)?;

        let gutter_width: usize = std::cmp::max(
            (self.span.end().page.line as f32).log10().ceil() as usize, 1);
        let riser_width = 3;

        write_gutter(f, "", gutter_width)?;
        writeln!(f, "");
        for line in self.span.split_lines() {
            write_gutter(f,
                line.start().page.line,
                gutter_width)?;
            write_riser(f,
                vec![RiserSymbol::Bar, RiserSymbol::Bar].into_iter(),
                riser_width)?;
            write_source_ln(f,
                line)?;
        }
        write_gutter(f, "", gutter_width)?;
        writeln!(f, "");
        Ok(())
    }
}


fn write_gutter<G>(
    f: &mut std::fmt::Formatter<'_>,
    value: G,
    width: usize)
    -> std::fmt::Result
    where G: std::fmt::Display,
{
    write!(f, "{:>width$} | ",
        value,
        width=width)
}

fn write_riser<R>(
    f: &mut std::fmt::Formatter<'_>,
    risers: R,
    width: usize)
    -> std::fmt::Result
    where R: Iterator<Item=RiserSymbol>,
{
    let mut i = 0;
    for riser in risers {
        match riser {
            RiserSymbol::Empty => write!(f, " ")?,
            RiserSymbol::Bar   => write!(f, "|")?,
            RiserSymbol::Point => write!(f, "/")?,
        }
        i += 1;
    }
    for i in i..width {
        write!(f, " ")?;
    }
    Ok(())
}

enum RiserSymbol {
    Empty,
    Bar,
    Point,
}

fn write_source_ln<'text, Nl>(
    f: &mut std::fmt::Formatter<'_>,
    span: Span<'text, Nl>)
    -> std::fmt::Result
    where Nl: NewLine,
{
    writeln!(f, " {}", span.widen_to_line().text())
}

fn write_highlight_ln<'text,'msg,  Nl>(
    f: &mut std::fmt::Formatter<'_>,
    span: Span<'text, Nl>,
    highlight_sym: HighlightSymbol,
    message: &'msg str)
    -> std::fmt::Result
{
    let leader_width = span.start().page.column;
    for _ in 0..leader_width {
        write!(f, "_")?;
    }

    let underline_width = span.end().page.column - span.start().page.column;
    match highlight_sym {
        HighlightSymbol::Dash
            => write!(f, "{:-width$}", "", width=underline_width)?,
        HighlightSymbol::Hat
            => write!(f, "{:^width$}", "", width=underline_width)?,
    }

    writeln!(f, "{}", message)
}

enum HighlightSymbol {
    Dash,
    Hat,
}

////////////////////////////////////////////////////////////////////////////////
// Highlight
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Highlight<'text, 'msg, Nl> {
    span: Span<'text, Nl>,
    start_message: Option<&'msg str>,
    end_message: Option<&'msg str>,
    color: (),
    underline: (),
}

impl<'text, 'msg, Nl> Highlight<'text, 'msg, Nl> {

}
