use mlua::prelude::*;

use crate::fmt::ErrorComponents;

fn new_lua_result() -> LuaResult<()> {
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

// Tests for error context stack
mod context {
    use super::*;

    #[test]
    fn preserves_original() {
        let lua_error = new_lua_result().context("additional context").unwrap_err();
        let components = ErrorComponents::from(lua_error);

        assert_eq!(components.messages()[0], "additional context");
        assert_eq!(components.messages()[1], "oh no, a runtime error");
    }

    #[test]
    fn preserves_levels() {
        // NOTE: The behavior in mlua is to preserve a single level of context
        // and not all levels (context gets replaced on each call to `context`)
        let lua_error = new_lua_result()
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
        let lua_error = new_lua_result().unwrap_err();
        let components = ErrorComponents::from(lua_error);

        assert_eq!(components.messages()[0], "oh no, a runtime error");
    }

    #[test]
    fn stack_begin_end() {
        let lua_error = new_lua_result().unwrap_err();
        let formatted = format!("{}", ErrorComponents::from(lua_error));

        assert!(formatted.contains("Stack Begin"));
        assert!(formatted.contains("Stack End"));
    }

    #[test]
    fn stack_lines() {
        let lua_error = new_lua_result().unwrap_err();
        let components = ErrorComponents::from(lua_error);

        let mut lines = components.trace().unwrap().lines().iter();
        let line_1 = lines.next().unwrap().to_string();
        let line_2 = lines.next().unwrap().to_string();
        assert!(lines.next().is_none());

        assert_eq!(line_1, "Script '[C]' - function 'f'");
        assert_eq!(line_2, "Script 'chunk_name', Line 1");
    }
}
