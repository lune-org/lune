use std::sync::LazyLock;

use console::Style;

pub static COLOR_GREEN: LazyLock<Style> = LazyLock::new(|| Style::new().green());
pub static COLOR_YELLOW: LazyLock<Style> = LazyLock::new(|| Style::new().yellow());
pub static COLOR_MAGENTA: LazyLock<Style> = LazyLock::new(|| Style::new().magenta());
pub static COLOR_CYAN: LazyLock<Style> = LazyLock::new(|| Style::new().cyan());

pub static STYLE_DIM: LazyLock<Style> = LazyLock::new(|| Style::new().dim());
