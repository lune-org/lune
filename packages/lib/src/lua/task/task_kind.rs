use std::fmt;

/// Enum representing different kinds of tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskKind {
    Instant,
    Deferred,
    Future,
}

#[allow(dead_code)]
impl TaskKind {
    pub fn is_instant(&self) -> bool {
        *self == Self::Instant
    }

    pub fn is_deferred(&self) -> bool {
        *self == Self::Deferred
    }

    pub fn is_blocking(&self) -> bool {
        *self != Self::Future
    }

    pub fn is_future(&self) -> bool {
        *self == Self::Future
    }
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &'static str = match self {
            TaskKind::Instant => "Instant",
            TaskKind::Deferred => "Deferred",
            TaskKind::Future => "Future",
        };
        write!(f, "{name}")
    }
}
