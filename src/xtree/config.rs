use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Json,
    Markdown,
    Prompt,
    Fingerprint,
}

#[derive(Clone, Copy, Debug)]
pub enum SortMode {
    Name,
    Size,
    Time,
}

#[derive(Debug)]
pub struct Config {
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
    pub show_size: bool,
    pub show_info: bool,
    pub dir_only: bool,
    pub target_dir: PathBuf,
    pub excludes: Vec<String>,
    pub cat_exts: Vec<String>,
    pub grep_pattern: Option<String>,
    pub use_gitignore: bool,
    pub show_stats: bool,
    pub prompt_mode: bool,
    pub highlight_big: bool,
    pub big_threshold: u64,
    pub find_dupes: bool,
    pub audit_mode: bool,
    pub sort_mode: SortMode,
    pub group_by_ext: bool,
    pub focus_exts: Vec<String>,
    pub show_tests: bool,
    pub resolve_symlinks: bool,
    pub no_clip: bool,
    pub clip: usize,
    pub show_file_count: bool,
    pub use_treeignore: bool,
    pub show_icons: bool,
    pub no_report: bool,
    pub theme: String,
    pub mode: Mode,
    pub help: bool,
    pub tldr: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_depth: None,
            show_hidden: false,
            show_size: true,
            show_info: false,
            dir_only: false,
            target_dir: PathBuf::from("."),
            excludes: Vec::new(),
            cat_exts: Vec::new(),
            grep_pattern: None,
            use_gitignore: true,
            show_stats: false,
            prompt_mode: false,
            highlight_big: false,
            big_threshold: 5 * 1024 * 1024,
            find_dupes: false,
            audit_mode: false,
            sort_mode: SortMode::Name,
            group_by_ext: false,
            focus_exts: Vec::new(),
            show_tests: false,
            resolve_symlinks: false,
            no_clip: false,
            clip: 100,
            show_file_count: false,
            use_treeignore: true,
            show_icons: true,
            no_report: false,
            theme: "default".to_string(),
            mode: Mode::Normal,
            help: false,
            tldr: false,
        }
    }
}

impl Config {
    pub fn parse(args: &[OsString]) -> Result<Self, String> {
        let mut cfg = Config::default();
        let mut it = args.iter().peekable();

        while let Some(arg) = it.next() {
            let s = arg.to_string_lossy();
            match s.as_ref() {
                "-h" | "--help" => cfg.help = true,
                "--tldr" => cfg.tldr = true,
                "-L" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "-L requires a depth value".to_string())?;
                    cfg.max_depth = Some(
                        v.to_string_lossy()
                            .parse::<usize>()
                            .map_err(|_| "invalid depth for -L".to_string())?,
                    );
                }
                "-e" | "--exclude" | "-I" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "--exclude/-I requires a pattern".to_string())?;
                    cfg.excludes.push(v.to_string_lossy().to_string());
                }
                "-a" => cfg.show_hidden = true,
                "-d" => cfg.dir_only = true,
                "-s" => cfg.show_size = false,
                "-i" => cfg.show_info = true,
                "-c" | "--cat" => {
                    while let Some(next) = it.peek() {
                        let nexts = next.to_string_lossy();
                        if nexts.starts_with('-') {
                            break;
                        }
                        cfg.cat_exts
                            .push(nexts.trim_start_matches('.').to_ascii_lowercase());
                        let _ = it.next();
                    }
                }
                "-g" | "--grep" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "--grep requires a pattern".to_string())?;
                    cfg.grep_pattern = Some(v.to_string_lossy().to_string());
                }
                "--no-git" => cfg.use_gitignore = false,
                "--no-treeignore" => cfg.use_treeignore = false,
                "--no-icon" => cfg.show_icons = false,
                "--stats" => cfg.show_stats = true,
                "--md" => cfg.mode = Mode::Markdown,
                "--json" => cfg.mode = Mode::Json,
                "--clip" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "--clip requires a line count".to_string())?;
                    cfg.clip = v
                        .to_string_lossy()
                        .parse::<usize>()
                        .map_err(|_| "invalid line count for --clip".to_string())?;
                }
                "--nc" | "--no-clip" => cfg.no_clip = true,
                "--prompt" => {
                    cfg.prompt_mode = true;
                    cfg.mode = Mode::Prompt;
                }
                "--big" => cfg.highlight_big = true,
                "--dupes" => cfg.find_dupes = true,
                "--audit" => cfg.audit_mode = true,
                "--sort" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "--sort requires name|size|time".to_string())?;
                    cfg.sort_mode = match v.to_string_lossy().as_ref() {
                        "name" => SortMode::Name,
                        "size" => SortMode::Size,
                        "time" => SortMode::Time,
                        _ => return Err("--sort supports: name | size | time".to_string()),
                    };
                }
                "--group" => cfg.group_by_ext = true,
                "--focus" => {
                    while let Some(next) = it.peek() {
                        let nexts = next.to_string_lossy();
                        if nexts.starts_with('-') {
                            break;
                        }
                        cfg.focus_exts
                            .push(nexts.trim_start_matches('.').to_ascii_lowercase());
                        let _ = it.next();
                    }
                }
                "--tests" => cfg.show_tests = true,
                "--fingerprint" => cfg.mode = Mode::Fingerprint,
                "--resolve" => cfg.resolve_symlinks = true,
                "--theme" => {
                    let v = it
                        .next()
                        .ok_or_else(|| "--theme requires a theme name".to_string())?;
                    cfg.theme = v.to_string_lossy().to_string();
                }
                "--count" => cfg.show_file_count = true,
                "--noreport" => cfg.no_report = true,
                "--" => break,
                _ if s.starts_with('-') => return Err(format!("Unknown option: {s}")),
                _ => cfg.target_dir = PathBuf::from(s.to_string()),
            }
        }

        Ok(cfg)
    }
}
