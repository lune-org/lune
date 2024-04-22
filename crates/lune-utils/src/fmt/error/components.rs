use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use console::style;
use mlua::prelude::*;
use once_cell::sync::Lazy;

use super::StackTrace;

static STYLED_STACK_BEGIN: Lazy<String> = Lazy::new(|| {
    format!(
        "{}{}{}",
        style("[").dim(),
        style("Stack Begin").blue(),
        style("]").dim()
    )
});

static STYLED_STACK_END: Lazy<String> = Lazy::new(|| {
    format!(
        "{}{}{}",
        style("[").dim(),
        style("Stack End").blue(),
        style("]").dim()
    )
});

/**
    Error components parsed from a [`LuaError`].

    Can be used to display a human-friendly error message
    and stack trace, in the following Roblox-inspired format:

    ```plaintext
    Error message
    [Stack Begin]
        Stack trace line
        Stack trace line
        Stack trace line
    [Stack End]
    ```
*/
#[derive(Debug, Default, Clone)]
pub struct ErrorComponents {
    messages: Vec<String>,
    trace: Option<StackTrace>,
}

impl ErrorComponents {
    /**
        Returns the error messages.
    */
    #[must_use]
    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    /**
        Returns the stack trace, if it exists.
    */
    #[must_use]
    pub fn trace(&self) -> Option<&StackTrace> {
        self.trace.as_ref()
    }

    /**
        Returns `true` if the error has a non-empty stack trace.

        Note that a trace may still *exist*, but it may be empty.
    */
    #[must_use]
    pub fn has_trace(&self) -> bool {
        self.trace
            .as_ref()
            .is_some_and(|trace| !trace.lines().is_empty())
    }
}

impl fmt::Display for ErrorComponents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for message in self.messages() {
            writeln!(f, "{message}")?;
        }
        if self.has_trace() {
            let trace = self.trace.as_ref().unwrap();
            writeln!(f, "{}", *STYLED_STACK_BEGIN)?;
            for line in trace.lines() {
                writeln!(f, "\t{line}")?;
            }
            writeln!(f, "{}", *STYLED_STACK_END)?;
        }
        Ok(())
    }
}

impl From<LuaError> for ErrorComponents {
    fn from(error: LuaError) -> Self {
        fn lua_error_message(e: &LuaError) -> String {
            if let LuaError::RuntimeError(s) = e {
                s.to_string()
            } else {
                e.to_string()
            }
        }

        fn lua_stack_trace(source: &str) -> Option<StackTrace> {
            // FUTURE: Preserve a parsing error here somehow?
            // Maybe we can emit parsing errors using tracing?
            StackTrace::from_str(source).ok()
        }

        // Extract any additional "context" messages before the actual error(s)
        // The Arc is necessary here because mlua wraps all inner errors in an Arc
        let mut error = Arc::new(error);
        let mut messages = Vec::new();
        while let LuaError::WithContext {
            ref context,
            ref cause,
        } = *error
        {
            messages.push(context.to_string());
            error = cause.clone();
        }

        // We will then try to extract any stack trace
        let trace = if let LuaError::CallbackError {
            ref traceback,
            ref cause,
        } = *error
        {
            messages.push(lua_error_message(cause));
            lua_stack_trace(traceback)
        } else if let LuaError::RuntimeError(ref s) = *error {
            // NOTE: Runtime errors may include tracebacks, but they're
            // joined with error messages, so we need to split them out
            if let Some(pos) = s.find("stack traceback:") {
                let (message, traceback) = s.split_at(pos);
                messages.push(message.trim().to_string());
                lua_stack_trace(traceback)
            } else {
                messages.push(s.to_string());
                None
            }
        } else {
            messages.push(lua_error_message(&error));
            None
        };

        ErrorComponents { messages, trace }
    }
}
