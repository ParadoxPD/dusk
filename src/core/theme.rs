use crate::core::style::Style;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub accent: &'static str,
    pub ok: &'static str,
    pub warn: &'static str,
    pub subtle: &'static str,
    pub info: &'static str,
    pub number: &'static str,
    pub title: &'static str,
    pub reset: &'static str,
}

pub const THEMES: [Theme; 14] = [
    Theme {
        name: "default",
        accent: "\x1b[1;94m",
        ok: "\x1b[1;92m",
        warn: "\x1b[1;91m",
        subtle: "\x1b[1;96m",
        info: "\x1b[1;97m",
        number: "\x1b[1;95m",
        title: "\x1b[1;95m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "nord",
        accent: "\x1b[38;5;81m",
        ok: "\x1b[38;5;121m",
        warn: "\x1b[38;5;204m",
        subtle: "\x1b[38;5;117m",
        info: "\x1b[38;5;159m",
        number: "\x1b[38;5;111m",
        title: "\x1b[1;38;5;153m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "gruvbox",
        accent: "\x1b[38;5;214m",
        ok: "\x1b[38;5;148m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;220m",
        info: "\x1b[38;5;229m",
        number: "\x1b[38;5;180m",
        title: "\x1b[1;38;5;214m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "dracula",
        accent: "\x1b[38;5;141m",
        ok: "\x1b[38;5;84m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;147m",
        info: "\x1b[38;5;225m",
        number: "\x1b[38;5;213m",
        title: "\x1b[1;38;5;213m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "solarized",
        accent: "\x1b[38;5;136m",
        ok: "\x1b[38;5;64m",
        warn: "\x1b[38;5;166m",
        subtle: "\x1b[38;5;37m",
        info: "\x1b[38;5;230m",
        number: "\x1b[38;5;74m",
        title: "\x1b[1;38;5;136m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "catppuccin",
        accent: "\x1b[38;5;183m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;210m",
        subtle: "\x1b[38;5;153m",
        info: "\x1b[38;5;225m",
        number: "\x1b[38;5;147m",
        title: "\x1b[1;38;5;183m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "tokyonight",
        accent: "\x1b[38;5;111m",
        ok: "\x1b[38;5;121m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;117m",
        info: "\x1b[38;5;189m",
        number: "\x1b[38;5;147m",
        title: "\x1b[1;38;5;147m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "onedark-pro",
        accent: "\x1b[38;5;75m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;110m",
        info: "\x1b[38;5;188m",
        number: "\x1b[38;5;152m",
        title: "\x1b[1;38;5;75m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "monokai",
        accent: "\x1b[38;5;81m",
        ok: "\x1b[38;5;118m",
        warn: "\x1b[38;5;197m",
        subtle: "\x1b[38;5;141m",
        info: "\x1b[38;5;227m",
        number: "\x1b[38;5;213m",
        title: "\x1b[1;38;5;227m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "kanagawa",
        accent: "\x1b[38;5;110m",
        ok: "\x1b[38;5;150m",
        warn: "\x1b[38;5;174m",
        subtle: "\x1b[38;5;180m",
        info: "\x1b[38;5;223m",
        number: "\x1b[38;5;109m",
        title: "\x1b[1;38;5;180m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "everforest",
        accent: "\x1b[38;5;108m",
        ok: "\x1b[38;5;142m",
        warn: "\x1b[38;5;167m",
        subtle: "\x1b[38;5;180m",
        info: "\x1b[38;5;223m",
        number: "\x1b[38;5;151m",
        title: "\x1b[1;38;5;108m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "rose-pine",
        accent: "\x1b[38;5;181m",
        ok: "\x1b[38;5;151m",
        warn: "\x1b[38;5;217m",
        subtle: "\x1b[38;5;146m",
        info: "\x1b[38;5;224m",
        number: "\x1b[38;5;182m",
        title: "\x1b[1;38;5;181m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "ayu",
        accent: "\x1b[38;5;215m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;151m",
        info: "\x1b[38;5;223m",
        number: "\x1b[38;5;179m",
        title: "\x1b[1;38;5;215m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "nightfox",
        accent: "\x1b[38;5;81m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;117m",
        info: "\x1b[38;5;189m",
        number: "\x1b[38;5;153m",
        title: "\x1b[1;38;5;81m",
        reset: "\x1b[0m",
    },
];

pub fn resolve(name: Option<&str>) -> Theme {
    let env_theme = std::env::var("DUSK_THEME").ok();
    let mut wanted = name.or(env_theme.as_deref()).unwrap_or("onedark-pro");
    if wanted == "onedark" {
        wanted = "onedark-pro";
    }
    THEMES
        .iter()
        .copied()
        .find(|theme| theme.name == wanted)
        .unwrap_or(THEMES[7])
}

pub fn plain() -> Theme {
    Theme {
        name: "plain",
        accent: "",
        ok: "",
        warn: "",
        subtle: "",
        info: "",
        number: "",
        title: "",
        reset: "",
    }
}

pub fn active(name: Option<&str>) -> Theme {
    let style = Style::for_stdout();
    if style.color { resolve(name) } else { plain() }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn resolves_onedark_alias_to_onedark_pro() {
        let t = resolve(Some("onedark"));
        assert_eq!(t.name, "onedark-pro");
    }

    #[test]
    fn unknown_theme_falls_back_to_default_theme() {
        let t = resolve(Some("unknown-theme"));
        assert_eq!(t.name, "onedark-pro");
    }
}
