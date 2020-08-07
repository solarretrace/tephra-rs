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
        let riser_width: usize = self.highlights
            .iter()
            .filter(|h| h.is_multiline())
            .count();
        let mut multi_split = MultiSplitLines::new(&self.highlights);

        write_source_info_line(f, gutter_width, self.source_name, self.span)?;

        // Write starting spacer line.
        write_gutter(f, "", gutter_width)?;
        writeln!(f, "");

        let mut end_spacer_needed = true;
        for (line_num, line_span) in self.span.split_lines()
            .map(|span| (span.start().page.line, span))
        {
            // println!("## LS {}", line_span);
            write_gutter(f, line_num, gutter_width)?;
            write_riser(f,
                multi_split.riser_data(line_num),
                riser_width,
                false)?;
            write_source_ln(f,
                line_span)?;
            end_spacer_needed = true;

            for line_data in multi_split.line_data(line_num) {
                // println!("## LD {:?}", line_data);
                write_gutter(f, "", gutter_width)?;
                write_riser(f,
                    multi_split.riser_data(line_num),
                    riser_width,
                    line_data.leader_line)?;
                write_highlight_ln(f,
                    line_data.span,
                    line_data.highlight_ul,
                    line_data.leader_line,
                    line_data.message_a)?;
                end_spacer_needed = false;
            }

        }
        if end_spacer_needed {
            write_gutter(f, "", gutter_width)?;
            writeln!(f, "");
        }
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
    width: usize,
    leader_line: bool)
    -> std::fmt::Result
    where R: Iterator<Item=RiserSymbol>,
{
    let mut i = 0;
    for riser in risers {
        match riser {
            RiserSymbol::Empty   => write!(f, " ")?,
            RiserSymbol::Bar     => write!(f, "|")?,
            RiserSymbol::UpPoint => write!(f, "/")?,
            RiserSymbol::DnPoint => write!(f, "\\")?,
        }
        i += 1;
    }
    for i in i..width {
        write!(f, " ")?;
    }
    if leader_line {
        write!(f, "_")?;
    } else {
        write!(f, " ")?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RiserSymbol {
    Empty,
    UpPoint,
    Bar,
    DnPoint,
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
        span.start().page,
        span.start().byte,
        width=gutter_width)
}

fn write_source_ln<'text, Nl>(
    f: &mut std::fmt::Formatter<'_>,
    span: Span<'text, Nl>)
    -> std::fmt::Result
    where Nl: NewLine,
{
    writeln!(f, "{}", span.widen_to_line().text())
}

fn write_highlight_ln<'text,'msg,  Nl>(
    f: &mut std::fmt::Formatter<'_>,
    span: Span<'text, Nl>,
    highlight_ul: HighlightUnderline,
    leader_line: bool,
    message: &'msg str)
    -> std::fmt::Result
{
    let leader_width = span.start().page.column;
    for _ in 0..leader_width {
        if leader_line {
            write!(f, "_")?;
        } else {
            write!(f, " ")?;
        }
    }

    let underline_width = std::cmp::max(
        span.end().page.column - span.start().page.column,
        1);
    // TODO: Fix underlining of tab/multi-width characters.
    for i in 0..underline_width {
        if leader_line && i < underline_width - 1 {
            write!(f, "_")?;
        } else {
            match highlight_ul {
                HighlightUnderline::Dash => write!(f, "-")?,
                HighlightUnderline::Hat  => write!(f, "^")?,
            }
        }
    }

    writeln!(f, " {}", message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HighlightUnderline {
    Dash,
    Hat,
}

////////////////////////////////////////////////////////////////////////////////
// Highlight
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Highlight<'text, 'msg, Nl> {
    span: Span<'text, Nl>,
    start_message: &'msg str,
    end_message: &'msg str,
    color: (),
    underline: HighlightUnderline,
}

impl<'text, 'msg, Nl> Highlight<'text, 'msg, Nl> {
    pub fn new(span: Span<'text, Nl>, end_message: &'msg str)
        -> Self
    {
        Highlight {
            span,
            start_message: "",
            end_message,
            color: (),
            underline: HighlightUnderline::Hat,
        }
    }

    pub fn with_start_message(mut self, start_message: &'msg str) -> Self {
        self.start_message = start_message;
        self
    }

    pub fn is_multiline(&self) -> bool {
        self.span.start().page.line != self.span.end().page.line
    }
}


////////////////////////////////////////////////////////////////////////////////
// MultiSplitLines
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
struct MultiSplitLines<'text, 'msg, 'hl, Nl> {
    highlights: &'hl [Highlight<'text, 'msg, Nl>],
    current: Vec<Option<Span<'text, Nl>>>,
    spans: Vec<SplitLines<'text, Nl>>,
}

impl<'text, 'msg, 'hl, Nl> MultiSplitLines<'text, 'msg, 'hl, Nl> 
    where
        'text: 'msg,
        Nl: NewLine,
{
    fn new(highlights: &'hl [Highlight<'text, 'msg, Nl>]) -> Self {
        let mut current = Vec::with_capacity(highlights.len());
        let mut spans = Vec::with_capacity(highlights.len());
        for span in highlights.iter().map(|hl| &hl.span) {
            let mut split = span.split_lines();
            current.push(split.next());
            spans.push(split);
        }
        MultiSplitLines {
            highlights,
            current,
            spans,
        }
    }
}

impl<'text, 'msg, 'hl, Nl> MultiSplitLines<'text, 'msg, 'hl, Nl> 
    where 'text: 'msg,
{
    fn riser_data(&self, line: usize) -> impl Iterator<Item=RiserSymbol> {
        let mut riser_data = Vec::with_capacity(self.current.len());
        for hl in self.highlights
            .iter()
            .filter(|hl| hl.is_multiline()) 
        {
            let start_line = hl.span.start().page.line;
            let end_line = hl.span.end().page.line;
            
            if start_line == end_line {
                riser_data.push(RiserSymbol::Empty);
            } else if line == start_line {
                if hl.span.start().page.column == 0 {
                    riser_data.push(RiserSymbol::UpPoint);
                } else {
                    riser_data.push(RiserSymbol::Empty);
                }
            } else if line == end_line {
                if hl.span.end().page.column == 0 {
                    riser_data.push(RiserSymbol::DnPoint);
                } else {
                    riser_data.push(RiserSymbol::Bar);

                }
            } else if line > start_line && line < end_line {
                riser_data.push(RiserSymbol::Bar);
            }
        }
        riser_data.into_iter()
    }

    fn line_data(&mut self, line: usize)
        -> impl Iterator<Item=HighlighLineData<'text, 'msg, Nl>>
        where Nl: NewLine,
    {
        let mut line_data = Vec::with_capacity(self.current.len());
        for (i, (curr, spans)) in self.current
            .iter_mut()
            .zip(self.spans.iter_mut())
            .enumerate()
        {
            match curr {
                Some(c) if c.start().page.line == line => {
                    let hl = &self.highlights[i];
                    let message_a = if hl.span.end().page.line == line {
                        hl.end_message
                    } else {
                        ""
                    };
                    let message_b = if hl.span.start().page.line == line {
                        hl.start_message
                    } else {
                        ""
                    };
                    let leader_line = hl.span.start().page.line != hl.span.end().page.line;
                    line_data.push(HighlighLineData {
                        span: c.clone(),
                        highlight_ul: hl.underline,
                        message_a,
                        message_b,
                        leader_line,
                    });
                    *curr = spans.next();
                },
                _ => (),
            }
        }
        line_data.into_iter()
    }
}

#[derive(Debug)]
struct HighlighLineData<'text, 'msg, Nl> {
    span: Span<'text, Nl>,
    highlight_ul: HighlightUnderline,
    message_a: &'msg str,
    message_b: &'msg str,
    leader_line: bool,
}
