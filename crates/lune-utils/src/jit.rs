#[derive(Debug, Clone, Copy, Default)]
pub struct JitStatus(bool);

impl JitStatus {
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self(enabled)
    }

    pub fn set_status(&mut self, enabled: bool) {
        self.0 = enabled;
    }

    #[must_use]
    pub fn enabled(self) -> bool {
        self.0
    }
}

impl From<JitStatus> for bool {
    fn from(val: JitStatus) -> Self {
        val.enabled()
    }
}

impl From<bool> for JitStatus {
    fn from(val: bool) -> Self {
        Self::new(val)
    }
}
