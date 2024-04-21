use mlua::prelude::*;

use lune_utils::TableBuilder;

mod context;
use context::RequireContext;

mod alias;
mod library;
mod path;

const REQUIRE_IMPL: &str = r"
return require(source(), ...)
";

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    lua.set_app_data(RequireContext::new());

    /*
        Require implementation needs a few workarounds:

        - Async functions run outside of the lua resumption cycle,
          so the current lua thread, as well as its stack/debug info
          is not available, meaning we have to use a normal function

        - Using the async require function directly in another lua function
          would mean yielding across the metamethod/c-call boundary, meaning
          we have to first load our two functions into a normal lua chunk
          and then load that new chunk into our final require function

        Also note that we inspect the stack at level 2:

        1. The current c / rust function
        2. The wrapper lua chunk defined above
        3. The lua chunk we are require-ing from
    */

    let require_fn = lua.create_async_function(require)?;
    let get_source_fn = lua.create_function(move |lua, (): ()| match lua.inspect_stack(2) {
        None => Err(LuaError::runtime(
            "Failed to get stack info for require source",
        )),
        Some(info) => match info.source().source {
            None => Err(LuaError::runtime(
                "Stack info is missing source for require",
            )),
            Some(source) => lua.create_string(source.as_bytes()),
        },
    })?;

    let require_env = TableBuilder::new(lua)?
        .with_value("source", get_source_fn)?
        .with_value("require", require_fn)?
        .build_readonly()?;

    lua.load(REQUIRE_IMPL)
        .set_name("require")
        .set_environment(require_env)
        .into_function()?
        .into_lua(lua)
}

async fn require<'lua>(
    lua: &'lua Lua,
    (source, path): (LuaString<'lua>, LuaString<'lua>),
) -> LuaResult<LuaMultiValue<'lua>> {
    let source = source
        .to_str()
        .into_lua_err()
        .context("Failed to parse require source as string")?
        .to_string();

    let path = path
        .to_str()
        .into_lua_err()
        .context("Failed to parse require path as string")?
        .to_string();

    let context = lua
        .app_data_ref()
        .expect("Failed to get RequireContext from app data");

    if let Some(builtin_name) = path.strip_prefix("@lune/").map(str::to_ascii_lowercase) {
        library::require(lua, &context, &builtin_name)
    } else if let Some(aliased_path) = path.strip_prefix('@') {
        let (alias, path) = aliased_path.split_once('/').ok_or(LuaError::runtime(
            "Require with custom alias must contain '/' delimiter",
        ))?;
        alias::require(lua, &context, &source, alias, path).await
    } else {
        path::require(lua, &context, &source, &path).await
    }
}
