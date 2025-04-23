#[derive(Debug, Clone, Copy, Default)]
pub struct JitEnablement(bool);

impl JitEnablement {
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

impl From<JitEnablement> for bool {
    fn from(val: JitEnablement) -> Self {
        val.enabled()
    }
}

impl From<bool> for JitEnablement {
    fn from(val: bool) -> Self {
        Self::new(val)
    }
}
