/**
    Configuration for formatting values.
*/
#[derive(Debug, Clone, Copy)]
pub struct ValueFormatConfig {
    pub(super) max_depth: usize,
    pub(super) colors_enabled: bool,
}

impl ValueFormatConfig {
    /**
        Creates a new config with default values.
    */
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Sets the maximum depth to which tables will be formatted.
    */
    #[must_use]
    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }

    /**
        Sets whether colors should be enabled.

        Colors are disabled by default.
    */
    #[must_use]
    pub fn with_colors_enabled(self, colors_enabled: bool) -> Self {
        Self {
            colors_enabled,
            ..self
        }
    }
}

impl Default for ValueFormatConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            colors_enabled: false,
        }
    }
}
