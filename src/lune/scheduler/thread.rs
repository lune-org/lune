use std::sync::Arc;

use mlua::prelude::*;
use rand::Rng;
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
pub struct SchedulerThreadId(u128);

impl SchedulerThreadId {
    fn gen() -> Self {
        // FUTURE: Use a faster rng here?
        Self(rand::thread_rng().gen())
    }
}

/**
    Container for registry keys that point to a thread and thread arguments.
*/
#[derive(Debug)]
pub(super) struct SchedulerThread {
    scheduler_id: SchedulerThreadId,
    thread: LuaOwnedThread,
    args: LuaRegistryKey,
}

impl SchedulerThread {
    /**
        Creates a new scheduler thread container from the given thread and arguments.

        May fail if an allocation error occurs, is not fallible otherwise.
    */
    pub(super) fn new<'lua>(
        lua: &'lua Lua,
        thread: LuaOwnedThread,
        args: LuaMultiValue<'lua>,
    ) -> LuaResult<Self> {
        let args_vec = args.into_vec();

        let args = lua
            .create_registry_value(args_vec)
            .context("Failed to store value in registry")?;

        Ok(Self {
            scheduler_id: SchedulerThreadId::gen(),
            thread,
            args,
        })
    }

    /**
        Extracts the inner thread and args from the container.
    */
    pub(super) fn into_inner(self, lua: &Lua) -> (LuaOwnedThread, LuaMultiValue<'_>) {
        let args_vec = lua
            .registry_value(&self.args)
            .expect("Failed to get thread args from registry");

        let args = LuaMultiValue::from_vec(args_vec);

        lua.remove_registry_value(self.args)
            .expect("Failed to remove thread args from registry");

        (self.thread, args)
    }

    /**
        Retrieves the unique, randomly generated id for this scheduler thread.
    */
    pub(super) fn id(&self) -> SchedulerThreadId {
        self.scheduler_id
    }
}
