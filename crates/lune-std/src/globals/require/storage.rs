use crate::{library::StandardLibrary, luaurc::RequireAlias};
use mlua::prelude::*;
use std::collections::HashMap;

/// The private struct that's stored in mlua's app data container
#[derive(Debug, Default)]
struct RequireStorageData<'a> {
    std: HashMap<&'a str, HashMap<&'a str, Box<dyn StandardLibrary>>>,
    std_cache: HashMap<RequireAlias, LuaRegistryKey>,
    cache: HashMap<&'a str, LuaRegistryKey>,
}

#[derive(Debug)]
pub struct RequireStorage {}

impl RequireStorage {
    pub fn init(lua: &Lua) -> LuaResult<()> {
        if lua.set_app_data(RequireStorageData::default()).is_some() {
            Err(LuaError::runtime("RequireStorage::init got called twice"))
        } else {
            Ok(())
        }
    }

    pub fn std_exists(lua: &Lua, alias: &str) -> LuaResult<bool> {
        let data_ref = lua
            .app_data_ref::<RequireStorageData>()
            .ok_or(LuaError::runtime("Couldn't find RequireStorageData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        Ok(data_ref.std.contains_key(alias))
    }

    pub fn require_std(lua: &Lua, require_alias: RequireAlias) -> LuaResult<LuaMultiValue<'_>> {
        let data_ref = lua
            .app_data_ref::<RequireStorageData>()
            .ok_or(LuaError::runtime("Couldn't find RequireStorageData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        if let Some(cached) = data_ref.std_cache.get(&require_alias) {
            return cached.into_lua(lua)?.into_lua_multi(lua);
        }

        let libraries =
            data_ref
                .std
                .get(&require_alias.alias.as_str())
                .ok_or(mlua::Error::runtime(format!(
                    "Alias '{}' does not point to a built-in standard library",
                    require_alias.alias
                )))?;

        let std = libraries
            .get(require_alias.path.as_str())
            .ok_or(mlua::Error::runtime(format!(
                "Library '{}' does not point to a member of '{}' standard libraries",
                require_alias.path, require_alias.alias
            )))?;

        let multi = std.module(lua)?;
        let mutli_clone = multi.clone();
        let multi_reg = lua.create_registry_value(mutli_clone.into_vec())?;

        let mut data = lua
        .app_data_mut::<RequireStorageData>()
        .ok_or(LuaError::runtime("Couldn't find RequireStorageData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        data.std_cache.insert(require_alias, multi_reg);

        Ok(multi)
    }

    pub fn inject_std(
        lua: &Lua,
        alias: &'static str,
        std: impl StandardLibrary + 'static,
    ) -> LuaResult<()> {
        let mut data = lua
            .app_data_mut::<RequireStorageData>()
            .ok_or(LuaError::runtime("Couldn't find RequireStorageData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        if let Some(map) = data.std.get_mut(alias) {
            map.insert(std.name(), Box::new(std));
        } else {
            let mut map: HashMap<&str, Box<dyn StandardLibrary>> = HashMap::new();

            map.insert(std.name(), Box::new(std));

            data.std.insert(alias, map);
        };

        Ok(())
    }
}
