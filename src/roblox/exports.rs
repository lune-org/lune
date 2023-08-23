use mlua::prelude::*;

/**
    Trait for any item that should be exported as part of the `roblox` built-in library.

    This may be an enum or a struct that should export constants and/or constructs.

    ### Example usage

    ```rs
    use mlua::prelude::*;

    struct MyType(usize);

    impl MyType {
        pub fn new(n: usize) -> Self {
            Self(n)
        }
    }

    impl LuaExportsTable<'_> for MyType {
        const EXPORT_NAME: &'static str = "MyType";

        fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
            let my_type_new = |lua, n: Option<usize>| {
                Self::new(n.unwrap_or_default())
            };

            TableBuilder::new(lua)?
                .with_function("new", my_type_new)?
                .build_readonly()
        }
    }

    impl LuaUserData for MyType {
        // ...
    }
    ```
*/
pub trait LuaExportsTable<'lua> {
    const EXPORT_NAME: &'static str;

    fn create_exports_table(lua: &'lua Lua) -> LuaResult<LuaTable<'lua>>;
}

/**
    Exports a single item that implements the [`LuaExportsTable`] trait.

    Returns the name of the export, as well as the export table.

    ### Example usage

    ```rs
    let lua: mlua::Lua::new();

    let (name1, table1) = export::<Type1>(lua)?;
    let (name2, table2) = export::<Type2>(lua)?;
    ```
*/
pub fn export<'lua, T>(lua: &'lua Lua) -> LuaResult<(&'static str, LuaValue<'lua>)>
where
    T: LuaExportsTable<'lua>,
{
    Ok((
        T::EXPORT_NAME,
        <T as LuaExportsTable>::create_exports_table(lua)?.into_lua(lua)?,
    ))
}
