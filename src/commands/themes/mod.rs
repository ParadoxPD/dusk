use crate::core::style::Style;
use crate::core::theme::THEMES;

pub fn list() {
    let style = Style::for_stdout();
    println!("Available themes:");
    for theme in THEMES {
        if style.color {
            println!(
                "  - {}{}{}  {}●{} {}●{} {}●{}",
                theme.accent,
                theme.name,
                theme.reset,
                theme.accent,
                theme.reset,
                theme.ok,
                theme.reset,
                theme.warn,
                theme.reset
            );
        } else {
            println!("  - {}", theme.name);
        }
    }
}
