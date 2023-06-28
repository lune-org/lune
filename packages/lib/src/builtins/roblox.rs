use mlua::prelude::*;

use lune_roblox::{
    document::{Document, DocumentError, DocumentFormat, DocumentKind},
    instance::Instance,
};
use tokio::task;

use crate::lua::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let mut roblox_constants = Vec::new();
    let roblox_module = lune_roblox::module(lua)?;
    for pair in roblox_module.pairs::<LuaValue, LuaValue>() {
        roblox_constants.push(pair?);
    }
    TableBuilder::new(lua)?
        .with_values(roblox_constants)?
        .with_async_function("deserializePlace", deserialize_place)?
        .with_async_function("deserializeModel", deserialize_model)?
        .with_async_function("serializePlace", serialize_place)?
        .with_async_function("serializeModel", serialize_model)?
        .with_async_function("getAuthCookie", get_auth_cookie)?
        .build_readonly()
}

async fn deserialize_place<'lua>(
    lua: &'lua Lua,
    contents: LuaString<'lua>,
) -> LuaResult<LuaValue<'lua>> {
    let bytes = contents.as_bytes().to_vec();
    let fut = task::spawn_blocking(move || {
        let doc = Document::from_bytes(bytes, DocumentKind::Place)?;
        let data_model = doc.into_data_model_instance()?;
        Ok::<_, DocumentError>(data_model)
    });
    fut.await.map_err(LuaError::external)??.into_lua(lua)
}

async fn deserialize_model<'lua>(
    lua: &'lua Lua,
    contents: LuaString<'lua>,
) -> LuaResult<LuaValue<'lua>> {
    let bytes = contents.as_bytes().to_vec();
    let fut = task::spawn_blocking(move || {
        let doc = Document::from_bytes(bytes, DocumentKind::Model)?;
        let instance_array = doc.into_instance_array()?;
        Ok::<_, DocumentError>(instance_array)
    });
    fut.await.map_err(LuaError::external)??.into_lua(lua)
}

async fn serialize_place<'lua>(
    lua: &'lua Lua,
    (data_model, as_xml): (LuaUserDataRef<'lua, Instance>, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    let data_model = (*data_model).clone();
    let fut = task::spawn_blocking(move || {
        let doc = Document::from_data_model_instance(data_model)?;
        let bytes = doc.to_bytes_with_format(match as_xml {
            Some(true) => DocumentFormat::Xml,
            _ => DocumentFormat::Binary,
        })?;
        Ok::<_, DocumentError>(bytes)
    });
    let bytes = fut.await.map_err(LuaError::external)??;
    lua.create_string(bytes)
}

async fn serialize_model<'lua>(
    lua: &'lua Lua,
    (instances, as_xml): (Vec<LuaUserDataRef<'lua, Instance>>, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    let instances = instances.iter().map(|i| (*i).clone()).collect();
    let fut = task::spawn_blocking(move || {
        let doc = Document::from_instance_array(instances)?;
        let bytes = doc.to_bytes_with_format(match as_xml {
            Some(true) => DocumentFormat::Xml,
            _ => DocumentFormat::Binary,
        })?;
        Ok::<_, DocumentError>(bytes)
    });
    let bytes = fut.await.map_err(LuaError::external)??;
    lua.create_string(bytes)
}

async fn get_auth_cookie(_: &Lua, raw: Option<bool>) -> LuaResult<Option<String>> {
    task::spawn_blocking(move || {
        if matches!(raw, Some(true)) {
            Ok(rbx_cookie::get_value())
        } else {
            Ok(rbx_cookie::get())
        }
    })
    .await
    .map_err(LuaError::external)?
}
