#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SchedulerMessage {
    ExitCodeSet,
    PushedLuaThread,
    SpawnedLuaFuture,
    SpawnedBackgroundFuture,
}

impl SchedulerMessage {
    pub fn should_break_futures(self) -> bool {
        matches!(self, Self::ExitCodeSet | Self::PushedLuaThread)
    }

    pub fn should_break_lua_futures(self) -> bool {
        self.should_break_futures() || matches!(self, Self::SpawnedBackgroundFuture)
    }

    pub fn should_break_background_futures(self) -> bool {
        self.should_break_futures() || matches!(self, Self::SpawnedLuaFuture)
    }
}
