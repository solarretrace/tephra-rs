////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stubbed tracing module.
////////////////////////////////////////////////////////////////////////////////


/// Stub for `tracing::span`.
#[macro_export]
macro_rules! span {
    // NOTE: $level is used here to avoid unused variable warnings.
    ($level:expr, $($a:tt) *) => {{ $level; $crate::Span }};
}

/// Stub for `tracing::event`.
#[macro_export]
macro_rules! event {
    // NOTE: $level is used here to avoid unused variable warnings.
    ($level:expr, $($a:tt) *) => { $level };
}

/// Stub for `tracing::Level`.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Level(LevelInner);

impl Level {
    /// Stub for `tracing::Level::TRACE`.
    pub const TRACE: Self = Self(LevelInner::Trace);
    /// Stub for `tracing::Level::DEBUG`.
    pub const DEBUG: Self = Self(LevelInner::Debug);
    /// Stub for `tracing::Level::INFO`.
    pub const INFO: Self = Self(LevelInner::Info);
    /// Stub for `tracing::Level::WARN`.
    pub const WARN: Self = Self(LevelInner::Warn);
    /// Stub for `tracing::Level::ERROR`.
    pub const ERROR: Self = Self(LevelInner::Error);
}

/// Internal representation of `Level`.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
#[repr(u8)]
enum LevelInner {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// Stub for `tracing::Span`.
#[derive(Debug, Clone, Copy)]
pub struct Span;

impl Span {
    /// Stub for `tracing::Span::entered`.
    #[must_use]
    pub fn entered(&self) -> Self { *self }

    /// Stub for `tracing::Span::enter`.
    pub fn enter(&self) {}

    /// Stub for `tracing::Span::exit`.
    pub fn exit(&self) {}
}
