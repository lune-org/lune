#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessJitEnablement {
    enabled: bool,
}

impl ProcessJitEnablement {
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn set_status(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[must_use]
    pub fn enabled(self) -> bool {
        self.enabled
    }
}

impl From<ProcessJitEnablement> for bool {
    fn from(val: ProcessJitEnablement) -> Self {
        val.enabled()
    }
}

impl From<bool> for ProcessJitEnablement {
    fn from(val: bool) -> Self {
        Self::new(val)
    }
}
