use std::env;

use atty::Stream;

#[derive(Clone, Copy)]
pub struct Style {
    pub color: bool,
    pub icons: bool,
}

impl Style {
    pub fn for_stdout() -> Self {
        let tty = atty::is(Stream::Stdout);
        let no_color = env::var_os("NO_COLOR").is_some();
        let term_dumb = env::var("TERM")
            .map(|v| v.eq_ignore_ascii_case("dumb"))
            .unwrap_or(false);

        let color = tty && !no_color && !term_dumb;
        let icons = tty;

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
