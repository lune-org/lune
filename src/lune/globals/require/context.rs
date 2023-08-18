use mlua::prelude::*;

const REGISTRY_KEY: &str = "RequireContext";

// TODO: Store current file path for each thread in
// this context somehow, as well as built-in libraries
#[derive(Clone)]
pub(super) struct RequireContext {
    pub(super) use_absolute_paths: bool,
}

impl RequireContext {
    pub fn new() -> Self {
        Self {
            // TODO: Set to false by default, load some kind of config
            // or env var to check if we should be using absolute paths
            use_absolute_paths: true,
        }
    }

    pub fn from_registry(lua: &Lua) -> Self {
        lua.named_registry_value(REGISTRY_KEY)
            .expect("Missing require context in lua registry")
    }

    pub fn insert_into_registry(self, lua: &Lua) {
        lua.set_named_registry_value(REGISTRY_KEY, self)
            .expect("Failed to insert RequireContext into registry");
    }
}

impl LuaUserData for RequireContext {}

impl<'lua> FromLua<'lua> for RequireContext {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(ctx) = ud.borrow::<RequireContext>() {
                return Ok(ctx.clone());
            }
        }
        unreachable!("RequireContext should only be used from registry")
    }
}
