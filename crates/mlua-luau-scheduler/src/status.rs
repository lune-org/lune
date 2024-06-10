#![allow(clippy::module_name_repetitions)]

/**
    The current status of a scheduler.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// The scheduler has not yet started running.
    NotStarted,
    /// The scheduler is currently running.
    Running,
    /// The scheduler has completed.
    Completed,
}

impl Status {
    #[must_use]
    pub const fn is_not_started(self) -> bool {
        matches!(self, Self::NotStarted)
    }

    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    #[must_use]
    pub const fn is_completed(self) -> bool {
        matches!(self, Self::Completed)
    }
}
