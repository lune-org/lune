use std::fmt;

use console::{style, Color};

#[derive(Debug, Clone, Copy)]
pub enum Label {
    Info,
    Warn,
    Error,
}

impl Label {
    /**
        Returns the name of the label in lowercase.
    */
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    /**
        Returns the color of the label.
    */
    #[must_use]
    pub fn color(&self) -> Color {
        match self {
            Self::Info => Color::Blue,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            style("[").dim(),
            style(self.name().to_ascii_uppercase()).fg(self.color()),
            style("]").dim()
        )
    }
}
