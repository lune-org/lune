use std::fmt;
use std::str::FromStr;

fn unwrap_braced_path(s: &str) -> &str {
    s.strip_prefix("[string \"")
        .and_then(|s2| s2.strip_suffix("\"]"))
        .unwrap_or(s)
}

fn parse_path(s: &str) -> Option<(&str, &str)> {
    let (path, after) = unwrap_braced_path(s).split_once(':')?;
    let path = unwrap_braced_path(path);

    // Remove line number after any found colon, this may
    // exist if the source path is from a rust source file
    let path = match path.split_once(':') {
        Some((before, _)) => before,
        None => path,
    };

    Some((path, after))
}

fn parse_function_name(s: &str) -> Option<&str> {
    s.strip_prefix("in function '")
        .and_then(|s| s.strip_suffix('\''))
}

fn parse_line_number(s: &str) -> (Option<usize>, &str) {
    match s.split_once(':') {
        Some((before, after)) => (before.parse::<usize>().ok(), after),
        None => (None, s),
    }
}

/**
    Source of a stack trace line parsed from a [`LuaError`].
*/
#[derive(Debug, Default, Clone, Copy)]
pub enum StackTraceSource {
    /// Error originated from a C / Rust function.
    C,
    /// Error originated from a Lua (user) function.
    #[default]
    Lua,
}

impl StackTraceSource {
    /**
        Returns `true` if the error originated from a C / Rust function, `false` otherwise.
    */
    #[must_use]
    pub const fn is_c(self) -> bool {
        matches!(self, Self::C)
    }

    /**
        Returns `true` if the error originated from a Lua (user) function, `false` otherwise.
    */
    #[must_use]
    pub const fn is_lua(self) -> bool {
        matches!(self, Self::Lua)
    }
}

/**
    Stack trace line parsed from a [`LuaError`].
*/
#[derive(Debug, Default, Clone)]
pub struct StackTraceLine {
    source: StackTraceSource,
    path: Option<String>,
    line_number: Option<usize>,
    function_name: Option<String>,
}

impl StackTraceLine {
    /**
        Returns the source of the stack trace line.
    */
    #[must_use]
    pub fn source(&self) -> StackTraceSource {
        self.source
    }

    /**
        Returns the path, if it exists.
    */
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /**
        Returns the line number, if it exists.
    */
    #[must_use]
    pub fn line_number(&self) -> Option<usize> {
        self.line_number
    }

    /**
        Returns the function name, if it exists.
    */
    #[must_use]
    pub fn function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    /**
        Returns `true` if the stack trace line contains no "useful" information, `false` otherwise.

        Useful information is determined as one of:

        - A path
        - A line number
        - A function name
    */
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.path.is_none() && self.line_number.is_none() && self.function_name.is_none()
    }
}

impl FromStr for StackTraceLine {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(after) = s.strip_prefix("[C]: ") {
            let function_name = parse_function_name(after).map(ToString::to_string);

            Ok(Self {
                source: StackTraceSource::C,
                path: None,
                line_number: None,
                function_name,
            })
        } else if let Some((path, after)) = parse_path(s) {
            let (line_number, after) = parse_line_number(after);
            let function_name = parse_function_name(after).map(ToString::to_string);

            Ok(Self {
                source: StackTraceSource::Lua,
                path: Some(path.to_string()),
                line_number,
                function_name,
            })
        } else {
            Err(String::from("unknown format"))
        }
    }
}

impl fmt::Display for StackTraceLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if matches!(self.source, StackTraceSource::C) {
            write!(f, "Script '[C]'")?;
        } else {
            write!(f, "Script '{}'", self.path.as_deref().unwrap_or("[?]"))?;
            if let Some(line_number) = self.line_number {
                write!(f, ", Line {line_number}")?;
            }
        }
        if let Some(function_name) = self.function_name.as_deref() {
            write!(f, " - function '{function_name}'")?;
        }
        Ok(())
    }
}

/**
    Stack trace parsed from a [`LuaError`].
*/
#[derive(Debug, Default, Clone)]
pub struct StackTrace {
    lines: Vec<StackTraceLine>,
}

impl StackTrace {
    /**
        Returns the individual stack trace lines.
    */
    #[must_use]
    pub fn lines(&self) -> &[StackTraceLine] {
        &self.lines
    }

    /**
        Returns the individual stack trace lines, mutably.
    */
    #[must_use]
    pub fn lines_mut(&mut self) -> &mut Vec<StackTraceLine> {
        &mut self.lines
    }
}

impl FromStr for StackTrace {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, after) = s
            .split_once("stack traceback:")
            .ok_or_else(|| String::from("missing 'stack traceback:' prefix"))?;
        let lines = after
            .trim()
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    None
                } else {
                    Some(line.parse())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StackTrace { lines })
    }
}
