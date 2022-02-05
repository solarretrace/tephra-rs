////////////////////////////////////////////////////////////////////////////////
// Tephra parser span library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Source text wrapper.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::ColumnMetrics;
use crate::Pos;


////////////////////////////////////////////////////////////////////////////////
// SourceText
////////////////////////////////////////////////////////////////////////////////
/// A positioned section of source text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceText<'text> {
    /// The source text.
    source: &'text str,
    /// The column metrics of the source text.
    metrics: ColumnMetrics,
    /// The position of the start of the source text.
    start: Pos,
}

impl<'text> SourceText<'text> {
    /// Constructs a new `SourceText` with the given start `Pos`.
    pub fn new(source: &'text str, start: Pos, metrics: ColumnMetrics) -> Self {
        SourceText {
            source,
            start,
            metrics,
        }
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn text(&self) -> &'text str {
        &self.source
    }

    pub fn start(&self) -> Pos {
        self.start
    }

    pub fn start_mut(&mut self) -> &mut Pos {
        &mut self.start
    }

    pub fn column_metrics(&self) -> ColumnMetrics {
        self.metrics
    }

    pub fn column_metrics_mut(&mut self) -> &mut ColumnMetrics {
        &mut self.metrics
    }
}


impl<'text> std::fmt::Display for SourceText<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.source.len() > 7 {
            write!(f, "{}...", &self.source[0..7])?;
        } else {
            write!(f, "{}...", &self.source[..])?;
        };

        write!(f, " ({}, {:?})", self.start, self.metrics)
    }
}

impl<'text> std::fmt::Debug for SourceText<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = if self.source.len() > 7 {
            format!("{}...", &self.source[0..7])
        } else {
            format!("{}...", &self.source[..])
        };
        f.debug_struct("SourceText")
            .field("source", &src)
            .field("start", &self.start)
            .field("metrics", &self.metrics)
            .finish()
    }
}
