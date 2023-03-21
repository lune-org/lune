use std::path::PathBuf;

use blocking::unblock;
use mlua::prelude::*;
use tokio::fs;

use lune_roblox::{
    document::{Document, DocumentError, DocumentFormat, DocumentKind},
    instance::Instance,
};

use crate::lua::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let mut roblox_constants = Vec::new();
    let roblox_module = lune_roblox::module(lua)?;
    for pair in roblox_module.pairs::<LuaValue, LuaValue>() {
        roblox_constants.push(pair?);
    }
    TableBuilder::new(lua)?
        .with_values(roblox_constants)?
        .with_async_function("readPlaceFile", read_place_file)?
        .with_async_function("readModelFile", read_model_file)?
        .with_async_function("writePlaceFile", write_place_file)?
        .with_async_function("writeModelFile", write_model_file)?
        .build_readonly()
}

fn parse_file_path(path: String) -> LuaResult<(PathBuf, DocumentFormat)> {
    let file_path = PathBuf::from(path);
    let file_ext = file_path
        .extension()
        .ok_or_else(|| {
            LuaError::RuntimeError(format!(
                "Missing file extension for file path: '{}'",
                file_path.display()
            ))
        })?
        .to_string_lossy();
    let doc_format = DocumentFormat::from_extension(&file_ext).ok_or_else(|| {
        LuaError::RuntimeError(format!(
            "Invalid file extension for writing place file: '{}'",
            file_ext
        ))
    })?;
    Ok((file_path, doc_format))
}

async fn read_place_file(lua: &Lua, path: String) -> LuaResult<LuaValue> {
    let bytes = fs::read(path).await.map_err(LuaError::external)?;
    let fut = unblock(move || {
        let doc = Document::from_bytes(bytes, DocumentKind::Place)?;
        let data_model = doc.into_data_model_instance()?;
        Ok::<_, DocumentError>(data_model)
    });
    fut.await?.to_lua(lua)
}

async fn read_model_file(lua: &Lua, path: String) -> LuaResult<LuaValue> {
    let bytes = fs::read(path).await.map_err(LuaError::external)?;
    let fut = unblock(move || {
        let doc = Document::from_bytes(bytes, DocumentKind::Model)?;
        let instance_array = doc.into_instance_array()?;
        Ok::<_, DocumentError>(instance_array)
    });
    fut.await?.to_lua(lua)
}

async fn write_place_file(_: &Lua, (path, data_model): (String, Instance)) -> LuaResult<()> {
    let (file_path, doc_format) = parse_file_path(path)?;
    let fut = unblock(move || {
        let doc = Document::from_data_model_instance(data_model)?;
        let bytes = doc.to_bytes_with_format(doc_format)?;
        Ok::<_, DocumentError>(bytes)
    });
    let bytes = fut.await?;
    fs::write(file_path, bytes)
        .await
        .map_err(LuaError::external)?;
    Ok(())
}

async fn write_model_file(_: &Lua, (path, instances): (String, Vec<Instance>)) -> LuaResult<()> {
    let (file_path, doc_format) = parse_file_path(path)?;
    let fut = unblock(move || {
        let doc = Document::from_instance_array(instances)?;
        let bytes = doc.to_bytes_with_format(doc_format)?;
        Ok::<_, DocumentError>(bytes)
    });
    let bytes = fut.await?;
    fs::write(file_path, bytes)
        .await
        .map_err(LuaError::external)?;
    Ok(())
}
