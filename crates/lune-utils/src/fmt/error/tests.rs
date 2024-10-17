use mlua::prelude::*;

use crate::fmt::ErrorComponents;

fn new_lua_runtime_error() -> LuaResult<()> {
    let lua = Lua::new();

    lua.globals()
        .set(
            "f",
            LuaFunction::wrap(|_, (): ()| {
                Err::<(), _>(LuaError::runtime("oh no, a runtime error"))
            }),
        )
        .unwrap();

    lua.load("f()").set_name("chunk_name").eval()
}

fn new_lua_script_error() -> LuaResult<()> {
    let lua = Lua::new();

    lua.load(
        "local function inner()\
        \n    error(\"oh no, a script error\")\
        \nend\
        \n\
        \nlocal function outer()\
        \n    inner()\
        \nend\
        \n\
        \nouter()\
        ",
    )
    .set_name("chunk_name")
    .eval()
}

// Tests for error context stack
mod context {
    use super::*;

    #[test]
    fn preserves_original() {
        let lua_error = new_lua_runtime_error()
            .context("additional context")
            .unwrap_err();
        let components = ErrorComponents::from(lua_error);

        assert_eq!(components.messages()[0], "additional context");
        assert_eq!(components.messages()[1], "oh no, a runtime error");
    }

    #[test]
    fn preserves_levels() {
        // NOTE: The behavior in mlua is to preserve a single level of context
        // and not all levels (context gets replaced on each call to `context`)
        let lua_error = new_lua_runtime_error()
            .context("level 1")
            .context("level 2")
            .context("level 3")
            .unwrap_err();
        let components = ErrorComponents::from(lua_error);

        assert_eq!(
            components.messages(),
            &["level 3", "oh no, a runtime error"]
        );
    }
}

// Tests for error components struct: separated messages + stack trace
mod error_components {
    use super::*;

    #[test]
    fn message() {
        let lua_error = new_lua_runtime_error().unwrap_err();
        let components = ErrorComponents::from(lua_error);

        assert_eq!(components.messages()[0], "oh no, a runtime error");
    }

    #[test]
    fn stack_begin_end() {
        let lua_error = new_lua_runtime_error().unwrap_err();
        let formatted = format!("{}", ErrorComponents::from(lua_error));

        assert!(formatted.contains("Stack Begin"));
        assert!(formatted.contains("Stack End"));
    }

    #[test]
    fn stack_lines() {
        let lua_error = new_lua_runtime_error().unwrap_err();
        let components = ErrorComponents::from(lua_error);

        let mut lines = components.trace().unwrap().lines().iter();
        let line_1 = lines.next().unwrap().to_string();
        let line_2 = lines.next().unwrap().to_string();
        assert!(lines.next().is_none());

        assert_eq!(line_1, "Script '[C]' - function 'f'");
        assert_eq!(line_2, "Script 'chunk_name', Line 1");
    }
}

// Tests for general formatting
mod general {
    use super::*;

    #[test]
    fn message_does_not_contain_location() {
        let lua_error = new_lua_script_error().unwrap_err();

        let components = ErrorComponents::from(lua_error);
        let trace = components.trace().unwrap();

        let first_message = components.messages().first().unwrap();
        let first_lua_stack_line = trace
            .lines()
            .iter()
            .find(|line| line.source().is_lua())
            .unwrap();

        let location_prefix = format!(
            "[string \"{}\"]:{}:",
            first_lua_stack_line.path().unwrap(),
            first_lua_stack_line.line_number().unwrap()
        );

        assert!(!first_message.starts_with(&location_prefix));
    }

    #[test]
    fn no_redundant_c_mentions() {
        let lua_error = new_lua_script_error().unwrap_err();

        let components = ErrorComponents::from(lua_error);
        let trace = components.trace().unwrap();

        let c_stack_lines = trace
            .lines()
            .iter()
            .filter(|line| line.source().is_c())
            .collect::<Vec<_>>();

        assert_eq!(c_stack_lines.len(), 1); // Just the "error" call
    }
}
