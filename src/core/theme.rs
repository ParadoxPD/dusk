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
        accent: "\x1b[1;34m",
        ok: "\x1b[1;32m",
        warn: "\x1b[1;31m",
        subtle: "\x1b[0;90m",
        info: "\x1b[1;36m",
        number: "\x1b[0;90m",
        title: "\x1b[1;95m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "nord",
        accent: "\x1b[38;5;81m",
        ok: "\x1b[38;5;150m",
        warn: "\x1b[38;5;204m",
        subtle: "\x1b[38;5;242m",
        info: "\x1b[38;5;109m",
        number: "\x1b[38;5;244m",
        title: "\x1b[1;38;5;143m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "gruvbox",
        accent: "\x1b[38;5;214m",
        ok: "\x1b[38;5;142m",
        warn: "\x1b[38;5;167m",
        subtle: "\x1b[38;5;243m",
        info: "\x1b[38;5;109m",
        number: "\x1b[38;5;243m",
        title: "\x1b[1;38;5;208m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "dracula",
        accent: "\x1b[38;5;212m",
        ok: "\x1b[38;5;84m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;239m",
        info: "\x1b[38;5;117m",
        number: "\x1b[38;5;239m",
        title: "\x1b[1;38;5;212m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "solarized",
        accent: "\x1b[38;5;136m",
        ok: "\x1b[38;5;64m",
        warn: "\x1b[38;5;166m",
        subtle: "\x1b[38;5;244m",
        info: "\x1b[38;5;37m",
        number: "\x1b[38;5;244m",
        title: "\x1b[1;38;5;136m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "catppuccin",
        accent: "\x1b[38;5;183m",
        ok: "\x1b[38;5;115m",
        warn: "\x1b[38;5;210m",
        subtle: "\x1b[38;5;246m",
        info: "\x1b[38;5;153m",
        number: "\x1b[38;5;246m",
        title: "\x1b[1;38;5;183m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "tokyonight",
        accent: "\x1b[38;5;111m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;244m",
        info: "\x1b[38;5;117m",
        number: "\x1b[38;5;244m",
        title: "\x1b[1;38;5;147m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "onedark",
        accent: "\x1b[38;5;75m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;242m",
        info: "\x1b[38;5;109m",
        number: "\x1b[38;5;242m",
        title: "\x1b[1;38;5;111m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "monokai",
        accent: "\x1b[38;5;81m",
        ok: "\x1b[38;5;148m",
        warn: "\x1b[38;5;197m",
        subtle: "\x1b[38;5;240m",
        info: "\x1b[38;5;141m",
        number: "\x1b[38;5;240m",
        title: "\x1b[1;38;5;227m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "kanagawa",
        accent: "\x1b[38;5;110m",
        ok: "\x1b[38;5;150m",
        warn: "\x1b[38;5;174m",
        subtle: "\x1b[38;5;242m",
        info: "\x1b[38;5;180m",
        number: "\x1b[38;5;242m",
        title: "\x1b[1;38;5;180m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "everforest",
        accent: "\x1b[38;5;108m",
        ok: "\x1b[38;5;142m",
        warn: "\x1b[38;5;167m",
        subtle: "\x1b[38;5;240m",
        info: "\x1b[38;5;109m",
        number: "\x1b[38;5;240m",
        title: "\x1b[1;38;5;108m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "rose-pine",
        accent: "\x1b[38;5;181m",
        ok: "\x1b[38;5;151m",
        warn: "\x1b[38;5;217m",
        subtle: "\x1b[38;5;245m",
        info: "\x1b[38;5;146m",
        number: "\x1b[38;5;245m",
        title: "\x1b[1;38;5;181m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "ayu",
        accent: "\x1b[38;5;215m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;242m",
        info: "\x1b[38;5;151m",
        number: "\x1b[38;5;242m",
        title: "\x1b[1;38;5;215m",
        reset: "\x1b[0m",
    },
    Theme {
        name: "nightfox",
        accent: "\x1b[38;5;75m",
        ok: "\x1b[38;5;114m",
        warn: "\x1b[38;5;203m",
        subtle: "\x1b[38;5;240m",
        info: "\x1b[38;5;117m",
        number: "\x1b[38;5;240m",
        title: "\x1b[1;38;5;81m",
        reset: "\x1b[0m",
    },
];

pub fn resolve(name: Option<&str>) -> Theme {
    let env_theme = std::env::var("DUSK_THEME").ok();
    let wanted = name.or(env_theme.as_deref()).unwrap_or("default");
    THEMES
        .iter()
        .copied()
        .find(|theme| theme.name == wanted)
        .unwrap_or(THEMES[0])
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
