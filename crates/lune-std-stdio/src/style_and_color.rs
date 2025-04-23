use std::str::FromStr;

use mlua::prelude::*;

const ESCAPE_SEQ_RESET: &str = "\x1b[0m";

/**
    A color kind supported by the `stdio` standard library.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorKind {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl ColorKind {
    pub const ALL: [Self; 9] = [
        Self::Reset,
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Blue,
        Self::Magenta,
        Self::Cyan,
        Self::White,
    ];

    /**
        Returns the human-friendly name of this color kind.
    */
    pub fn name(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Black => "black",
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Blue => "blue",
            Self::Magenta => "magenta",
            Self::Cyan => "cyan",
            Self::White => "white",
        }
    }

    /**
        Returns the ANSI escape sequence for the color kind.
    */
    pub fn ansi_escape_sequence(self) -> &'static str {
        match self {
            Self::Reset => ESCAPE_SEQ_RESET,
            Self::Black => "\x1b[30m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
        }
    }
}

impl FromStr for ColorKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_ascii_lowercase().as_str() {
            "reset" => Self::Reset,
            "black" => Self::Black,
            "red" => Self::Red,
            "green" => Self::Green,
            "yellow" => Self::Yellow,
            "blue" => Self::Blue,
            // NOTE: Previous versions of Lune had this color as "purple" instead
            // of "magenta", so we keep this here for backwards compatibility.
            "magenta" | "purple" => Self::Magenta,
            "cyan" => Self::Cyan,
            "white" => Self::White,
            _ => return Err(()),
        })
    }
}

impl FromLua for ColorKind {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = value {
            let s = s.to_str()?;
            match s.parse() {
                Ok(color) => Ok(color),
                Err(()) => Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "ColorKind".to_string(),
                    message: Some(format!(
                        "Invalid color kind '{s}'\nValid kinds are: {}",
                        Self::ALL
                            .iter()
                            .map(|kind| kind.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ColorKind".to_string(),
                message: None,
            })
        }
    }
}

/**
    A style kind supported by the `stdio` standard library.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKind {
    Reset,
    Bold,
    Dim,
}

impl StyleKind {
    pub const ALL: [Self; 3] = [Self::Reset, Self::Bold, Self::Dim];

    /**
        Returns the human-friendly name for this style kind.
    */
    pub fn name(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Bold => "bold",
            Self::Dim => "dim",
        }
    }

    /**
        Returns the ANSI escape sequence for this style kind.
    */
    pub fn ansi_escape_sequence(self) -> &'static str {
        match self {
            Self::Reset => ESCAPE_SEQ_RESET,
            Self::Bold => "\x1b[1m",
            Self::Dim => "\x1b[2m",
        }
    }
}

impl FromStr for StyleKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_ascii_lowercase().as_str() {
            "reset" => Self::Reset,
            "bold" => Self::Bold,
            "dim" => Self::Dim,
            _ => return Err(()),
        })
    }
}

impl FromLua for StyleKind {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = value {
            let s = s.to_str()?;
            match s.parse() {
                Ok(style) => Ok(style),
                Err(()) => Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "StyleKind".to_string(),
                    message: Some(format!(
                        "Invalid style kind '{s}'\nValid kinds are: {}",
                        Self::ALL
                            .iter()
                            .map(|kind| kind.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "StyleKind".to_string(),
                message: None,
            })
        }
    }
}
