use std::env;

use atty::Stream;

#[derive(Clone, Copy)]
pub struct Style {
    pub color: bool,
    pub icons: bool,
}

#[cfg(test)]
mod tests {
    use super::Style;

    #[test]
    fn paint_without_color_is_plain() {
        let style = Style {
            color: false,
            icons: false,
        };
        assert_eq!(style.paint("\x1b[31m", "hello"), "hello");
    }

    #[test]
    fn paint_with_color_wraps_ansi() {
        let style = Style {
            color: true,
            icons: true,
        };
        assert_eq!(style.paint("\x1b[31m", "x"), "\x1b[31mx\x1b[0m");
    }
}

impl Style {
    pub fn for_stdout() -> Self {
        let tty = atty::is(Stream::Stdout);
        let no_color = env::var_os("NO_COLOR").is_some();
        let force_color = env::var("DUSK_COLOR")
            .map(|v| v.eq_ignore_ascii_case("always"))
            .unwrap_or(false)
            || env::var("CLICOLOR_FORCE")
                .map(|v| v == "1")
                .unwrap_or(false);
        let term_dumb = env::var("TERM")
            .map(|v| v.eq_ignore_ascii_case("dumb"))
            .unwrap_or(false);

        let color = (tty && !no_color && !term_dumb) || force_color;
        let icons = tty || force_color;

        Self { color, icons }
    }

    pub fn paint(&self, ansi: &str, text: impl AsRef<str>) -> String {
        if self.color {
            format!("{ansi}{}\x1b[0m", text.as_ref())
        } else {
            text.as_ref().to_string()
        }
    }

    pub fn maybe_icon(&self, icon: &'static str) -> &'static str {
        if self.icons { icon } else { "" }
    }
}
