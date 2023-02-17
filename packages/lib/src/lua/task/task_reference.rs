use std::fmt;

use mlua::prelude::*;

use super::task_kind::TaskKind;

/// A lightweight, copyable struct that represents a
/// task in the scheduler and is accessible from Lua
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskReference {
    kind: TaskKind,
    guid: usize,
}

impl TaskReference {
    pub const fn new(kind: TaskKind, guid: usize) -> Self {
        Self { kind, guid }
    }

    pub const fn id(&self) -> usize {
        self.guid
    }
}

impl fmt::Display for TaskReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.guid == 0 {
            write!(f, "TaskReference(MAIN)")
        } else {
            write!(f, "TaskReference({} - {})", self.kind, self.guid)
        }
    }
}

impl LuaUserData for TaskReference {}
