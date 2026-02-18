mod analysis;
mod config;
mod help;
mod icons;
mod ignore;
mod outputs;
mod render;
mod theme;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::core::style::Style;
use analysis::{
    collect_duplicates, collect_stats, grouped_view, print_duplicates, print_fingerprint,
};
use config::{Config, Mode, SortMode};
use ignore::IgnoreMatcher;
use outputs::{print_json, print_markdown, print_prompt_dump};
use render::{print_tree, walk_files};
use theme::Theme;

pub fn run(args: &[OsString]) -> Result<(), String> {
    let cfg = Config::parse(args)?;

    if cfg.help {
        print!("{}", help::full_help(Some(cfg.theme.as_str())));
        return Ok(());
    }
    if cfg.tldr {
        print!("{}", help::tldr_help(Some(cfg.theme.as_str())));
        return Ok(());
    }

    if !cfg.target_dir.is_dir() {
        return Err(format!("Directory not found: {}", cfg.target_dir.display()));
    }

    let mut runtime = Runtime::new(cfg)?;
    runtime.execute()
}

struct Runtime {
    cfg: Config,
    root: PathBuf,
    theme: Theme,
    ignore: IgnoreMatcher,
}

impl Runtime {
    fn new(cfg: Config) -> Result<Self, String> {
        let root = cfg
            .target_dir
            .canonicalize()
            .map_err(|err| format!("failed to open target directory: {err}"))?;
        let mut theme = Theme::resolve(&cfg.theme);
        if cfg.prompt_mode || !Style::for_stdout().color {
            theme = Theme::plain();
        }
        let ignore = IgnoreMatcher::new(&root, &cfg)?;
        Ok(Self {
            cfg,
            root,
            theme,
            ignore,
        })
    }

    fn execute(&mut self) -> Result<(), String> {
        if self.cfg.group_by_ext {
            grouped_view(&self.root, &self.cfg, &self.ignore, &self.theme)?;
            return Ok(());
        }

        match self.cfg.mode {
            Mode::Json => {
                print_json(&self.root, &self.cfg, &self.ignore)?;
                Ok(())
            }
            Mode::Markdown => {
                print_markdown(&self.root, &self.cfg, &self.ignore)?;
                Ok(())
            }
            Mode::Prompt => print_prompt_dump(&self.root, &self.cfg, &self.ignore, &self.theme),
            Mode::Fingerprint => {
                let stats = collect_stats(&self.root, &self.cfg, &self.ignore)?;
                print_fingerprint(&self.root, &self.cfg, &self.theme, &self.ignore, &stats)?;
                if self.cfg.find_dupes {
                    let dupes = collect_duplicates(&self.root, &self.cfg, &self.ignore)?;
                    print_duplicates(&dupes, &self.cfg, &self.theme);
                }
                Ok(())
            }
            Mode::Normal => {
                println!(
                    "{}{}{}",
                    self.theme.dir,
                    self.root.display(),
                    self.theme.reset
                );
                let tree = print_tree(&self.root, &self.cfg, &self.ignore, &self.theme)?;
                if !self.cfg.no_report {
                    println!();
                    if self.cfg.dir_only {
                        println!(
                            "{}📁 {} directories{}",
                            self.theme.meta, tree.dir_count, self.theme.reset
                        );
                    } else {
                        println!(
                            "{}📁 {} directories, 📄 {} files{}",
                            self.theme.meta, tree.dir_count, tree.file_count, self.theme.reset
                        );
                    }
                }

                if self.cfg.show_stats {
                    let stats = collect_stats(&self.root, &self.cfg, &self.ignore)?;
                    analysis::print_stats(&stats, &self.theme);
                }

                if self.cfg.find_dupes {
                    let dupes = collect_duplicates(&self.root, &self.cfg, &self.ignore)?;
                    print_duplicates(&dupes, &self.cfg, &self.theme);
                }

                if self.cfg.grep_pattern.is_some()
                    || !self.cfg.cat_exts.is_empty()
                    || self.cfg.audit_mode
                {
                    let _ = walk_files(&self.root, &self.cfg, &self.ignore, |path| {
                        if let Some(pattern) = &self.cfg.grep_pattern {
                            render::print_grep(path, pattern, &self.theme);
                        }
                        if render::should_cat(path, &self.cfg) {
                            render::print_file_content(path, &self.cfg, &self.theme);
                        }
                        if self.cfg.audit_mode {
                            analysis::audit_file(path, &self.theme);
                        }
                    });
                }
                Ok(())
            }
        }
    }
}

fn _sort_key(path: &Path, mode: SortMode) -> (i64, String) {
    render::sort_key(path, mode)
}
