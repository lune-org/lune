use std::sync::Arc;

use mlua::prelude::*;
use tokio::sync::broadcast::Sender;

/**
    Type alias for a broadcast [`Sender`], which will
    broadcast the result and return values of a lua thread.

    The return values are stored in the lua registry as a
    `Vec<LuaValue<'_>>`, and the registry key pointing to
    those values will be sent using the broadcast sender.
*/
pub type SchedulerThreadSender = Sender<LuaResult<Arc<LuaRegistryKey>>>;

/**
    Unique, randomly generated id for a scheduler thread.
*/
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SchedulerThreadId(usize);

impl From<&LuaThread<'_>> for SchedulerThreadId {
    fn from(value: &LuaThread) -> Self {
        // HACK: We rely on the debug format of mlua
        // thread refs here, but currently this is the
        // only way to get a proper unique id using mlua
        let addr_string = format!("{value:?}");
        let addr = addr_string
            .strip_prefix("Thread(Ref(0x")
            .expect("Invalid thread address format - unknown prefix")
            .split_once(')')
            .map(|(s, _)| s)
            .expect("Invalid thread address format - missing ')'");
        let id = usize::from_str_radix(addr, 16)
            .expect("Failed to parse thread address as hexadecimal into usize");
        Self(id)
    }
}

/**
    Container for registry keys that point to a thread and thread arguments.
*/
#[derive(Debug)]
pub(super) struct SchedulerThread {
    thread_id: SchedulerThreadId,
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
    ) -> Self {
        let args_vec = args.into_vec();
        let thread_id = SchedulerThreadId::from(&thread);

        let key_thread = lua
            .create_registry_value(thread)
            .expect("Failed to store thread in registry - out of memory");
        let key_args = lua
            .create_registry_value(args_vec)
            .expect("Failed to store thread args in registry - out of memory");

        Self {
            thread_id,
            key_thread,
            key_args,
        }
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

    /**
        Retrieves the unique, randomly generated id for this scheduler thread.
    */
    pub(super) fn id(&self) -> SchedulerThreadId {
        self.thread_id
    }
}
