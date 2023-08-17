use mlua::prelude::*;

/**
    Container for registry keys that point to a thread and thread arguments.
*/
#[derive(Debug)]
pub(super) struct SchedulerThread {
    key_thread: LuaRegistryKey,
    key_args: LuaRegistryKey,
}

impl SchedulerThread {
    /**
        Creates a new scheduler thread container from the given thread and arguments.

        May fail if an allocation error occurs, is not fallible otherwise.
    */
    pub(super) fn new<'lua>(
        lua: &'lua Lua,
        thread: LuaThread<'lua>,
        args: LuaMultiValue<'lua>,
    ) -> LuaResult<Self> {
        let args_vec = args.into_vec();

        let key_thread = lua
            .create_registry_value(thread)
            .context("Failed to store value in registry")?;
        let key_args = lua
            .create_registry_value(args_vec)
            .context("Failed to store value in registry")?;

        Ok(Self {
            key_thread,
            key_args,
        })
    }

    /**
        Extracts the inner thread and args from the container.
    */
    pub(super) fn into_inner(self, lua: &Lua) -> (LuaThread<'_>, LuaMultiValue<'_>) {
        let thread = lua
            .registry_value(&self.key_thread)
            .expect("Failed to get thread from registry");
        let args_vec = lua
            .registry_value(&self.key_args)
            .expect("Failed to get thread args from registry");

        let args = LuaMultiValue::from_vec(args_vec);

        lua.remove_registry_value(self.key_thread)
            .expect("Failed to remove thread from registry");
        lua.remove_registry_value(self.key_args)
            .expect("Failed to remove thread args from registry");

        (thread, args)
    }
}
