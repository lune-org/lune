use std::fmt;

use console::{style, Color};

/**
    Label enum used for consistent output formatting throughout Lune.

    # Example usage

    ```rs
    use lune_utils::fmt::Label;

    println!("{} This is an info message", Label::Info);
    // [INFO] This is an info message

    println!("{} This is a warning message", Label::Warn);
    // [WARN] This is a warning message

    println!("{} This is an error message", Label::Error);
    // [ERROR] This is an error message
    ```
*/
#[derive(Debug, Clone, Copy)]
pub enum Label {
    Info,
    Warn,
    Error,
}

impl Label {
    /**
        Returns the name of the label in all uppercase.
    */
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
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
            style(self.name()).fg(self.color()),
            style("]").dim()
        )
    }
}
