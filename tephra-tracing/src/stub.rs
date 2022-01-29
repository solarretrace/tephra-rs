////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stubbed tracing module.
////////////////////////////////////////////////////////////////////////////////


#[macro_export]
macro_rules! span {
    ($level:expr, $($a:tt) *) => {{ $level; $crate::Span }};
}


#[macro_export]
macro_rules! event {
    ($level:expr, $($a:tt) *) => { $level };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Level(LevelInner);

impl Level {
    pub const TRACE: Level = Level(LevelInner::Trace);
    pub const DEBUG: Level = Level(LevelInner::Debug);
    pub const INFO: Level = Level(LevelInner::Info);
    pub const WARN: Level = Level(LevelInner::Warn);
    pub const ERROR: Level = Level(LevelInner::Error);
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
#[repr(u8)]
enum LevelInner {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}


#[derive(Debug, Clone, Copy)]
pub struct Span;

impl Span {
    pub fn entered(&self) {}
}
