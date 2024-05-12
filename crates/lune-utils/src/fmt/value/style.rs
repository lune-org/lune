use console::Style;
use once_cell::sync::Lazy;

pub static COLOR_GREEN: Lazy<Style> = Lazy::new(|| Style::new().green());
pub static COLOR_YELLOW: Lazy<Style> = Lazy::new(|| Style::new().yellow());
pub static COLOR_MAGENTA: Lazy<Style> = Lazy::new(|| Style::new().magenta());
pub static COLOR_CYAN: Lazy<Style> = Lazy::new(|| Style::new().cyan());

pub static STYLE_DIM: Lazy<Style> = Lazy::new(|| Style::new().dim());
