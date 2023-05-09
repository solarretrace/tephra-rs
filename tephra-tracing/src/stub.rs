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
    ($level:expr, $($a:tt) *) => { $crate::Span };
}

/// Stub for `tracing::event`.
#[macro_export]
macro_rules! event {
    ($level:expr, $($a:tt) *) => { };
}

/// Stub for `tracing::Level`.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Level(LevelInner);

impl Level {
    /// Stub for `tracing::Level::TRACE`.
    pub const TRACE: Level = Level(LevelInner::Trace);
    /// Stub for `tracing::Level::DEBUG`.
    pub const DEBUG: Level = Level(LevelInner::Debug);
    /// Stub for `tracing::Level::INFO`.
    pub const INFO: Level = Level(LevelInner::Info);
    /// Stub for `tracing::Level::WARN`.
    pub const WARN: Level = Level(LevelInner::Warn);
    /// Stub for `tracing::Level::ERROR`.
    pub const ERROR: Level = Level(LevelInner::Error);
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
    pub fn entered(&self) {}
}
