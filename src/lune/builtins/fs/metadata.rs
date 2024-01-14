use std::{
    fmt,
    fs::{FileType as StdFileType, Metadata as StdMetadata, Permissions as StdPermissions},
    io::Result as IoResult,
    str::FromStr,
    time::SystemTime,
};

use mlua::prelude::*;

use crate::lune::builtins::datetime::DateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsMetadataKind {
    None,
    File,
    Dir,
    Symlink,
}

impl fmt::Display for FsMetadataKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::File => "file",
                Self::Dir => "dir",
                Self::Symlink => "symlink",
            }
        )
    }
}

impl FromStr for FsMetadataKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "none" => Ok(Self::None),
            "file" => Ok(Self::File),
            "dir" => Ok(Self::Dir),
            "symlink" => Ok(Self::Symlink),
            _ => Err("Invalid metadata kind"),
        }
    }
}

impl From<StdFileType> for FsMetadataKind {
    fn from(value: StdFileType) -> Self {
        if value.is_file() {
            Self::File
        } else if value.is_dir() {
            Self::Dir
        } else if value.is_symlink() {
            Self::Symlink
        } else {
            panic!("Encountered unknown filesystem filetype")
        }
    }
}

impl<'lua> IntoLua<'lua> for FsMetadataKind {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        if self == Self::None {
            Ok(LuaValue::Nil)
        } else {
            self.to_string().into_lua(lua)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsPermissions {
    pub(crate) read_only: bool,
}

impl From<StdPermissions> for FsPermissions {
    fn from(value: StdPermissions) -> Self {
        Self {
            read_only: value.readonly(),
        }
    }
}

impl<'lua> IntoLua<'lua> for FsPermissions {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let tab = lua.create_table_with_capacity(0, 1)?;
        tab.set("readOnly", self.read_only)?;
        tab.set_readonly(true);
        Ok(LuaValue::Table(tab))
    }
}

#[derive(Debug, Clone)]
pub struct FsMetadata {
    pub(crate) kind: FsMetadataKind,
    pub(crate) exists: bool,
    pub(crate) created_at: Option<DateTime>,
    pub(crate) modified_at: Option<DateTime>,
    pub(crate) accessed_at: Option<DateTime>,
    pub(crate) permissions: Option<FsPermissions>,
}

impl FsMetadata {
    pub fn not_found() -> Self {
        Self {
            kind: FsMetadataKind::None,
            exists: false,
            created_at: None,
            modified_at: None,
            accessed_at: None,
            permissions: None,
        }
    }
}

impl<'lua> IntoLua<'lua> for FsMetadata {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let tab = lua.create_table_with_capacity(0, 6)?;
        tab.set("kind", self.kind)?;
        tab.set("exists", self.exists)?;
        tab.set("createdAt", self.created_at)?;
        tab.set("modifiedAt", self.modified_at)?;
        tab.set("accessedAt", self.accessed_at)?;
        tab.set("permissions", self.permissions)?;
        tab.set_readonly(true);
        Ok(LuaValue::Table(tab))
    }
}

impl From<StdMetadata> for FsMetadata {
    fn from(value: StdMetadata) -> Self {
        Self {
            kind: value.file_type().into(),
            exists: true,
            created_at: system_time_to_timestamp(value.created()),
            modified_at: system_time_to_timestamp(value.modified()),
            accessed_at: system_time_to_timestamp(value.accessed()),
            permissions: Some(FsPermissions::from(value.permissions())),
        }
    }
}

fn system_time_to_timestamp(res: IoResult<SystemTime>) -> Option<DateTime> {
    match res {
        Ok(t) => match t.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => DateTime::from_unix_timestamp_float(d.as_secs_f64()).ok(),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
