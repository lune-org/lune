use mlua::prelude::*;

/**
    Source of a stack trace line parsed from a [`LuaError`].
*/
#[derive(Debug, Clone, Copy)]
pub enum StackTraceSource {
    /// Error originated from a C function.
    C,
    /// Error originated from a Rust function.
    Rust,
    /// Error originated from [`mlua`].
    Mlua,
    /// Error originated from a Lua (user) function.
    User,
}

/**
    Stack trace line parsed from a [`LuaError`].
*/
#[derive(Debug, Clone)]
pub struct StackTraceLine {
    source: StackTraceSource,
    path: Option<String>,
    line_number: Option<usize>,
    function_name: Option<String>,
}

impl StackTraceLine {
    #[must_use]
    pub fn source(&self) -> StackTraceSource {
        self.source
    }

    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    #[must_use]
    pub fn line_number(&self) -> Option<usize> {
        self.line_number
    }

    #[must_use]
    pub fn function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }
}

/**
    Stack trace parsed from a [`LuaError`].
*/
#[derive(Debug, Clone)]
pub struct StackTrace {
    lines: Vec<StackTraceLine>,
}

impl StackTrace {
    #[must_use]
    pub fn lines(&self) -> &[StackTraceLine] {
        &self.lines
    }
}

/**
    Error components parsed from a [`LuaError`].
*/
#[derive(Debug, Clone)]
pub struct ErrorComponents {
    message: String,
    trace: StackTrace,
}

impl ErrorComponents {
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn trace(&self) -> &StackTrace {
        &self.trace
    }
}
