use std::cmp;
use std::process::Command;

use super::*;

impl App {
    pub(super) fn new(theme: Theme, style: Style) -> Self {
        Self {
            theme,
            style,
            pane: Pane::Files,
            tab: Tab::Workspace,
            input_mode: InputMode::None,
            input: String::new(),
            overlay: None,
            palette_query: String::new(),
            palette_selected: 0,
            status_msg: "Press ? for help".to_string(),
            branch: String::new(),
            upstream: None,
            files: Vec::new(),
            selected: 0,
            log_lines: Vec::new(),
            log_commits: Vec::new(),
            log_selected: 0,
            diff_lines: Vec::new(),
            commit_diff_lines: vec![
                "Select a commit in Graph tab (j/k), then open CommitDiff tab".to_string(),
            ],
            commit_diff_scroll: 0,
        }
    }

    pub(super) fn needs_cursor_blink(&self) -> bool {
        self.input_mode != InputMode::None || self.overlay == Some(Overlay::Palette)
    }

    pub(super) fn refresh(&mut self) -> Result<(), String> {
        self.branch = git_capture(&["branch", "--show-current"])?
            .trim()
            .to_string();
        self.upstream = git_capture(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let porcelain = git_capture(&["status", "--porcelain=1"])?;
        self.files = parse_porcelain(&porcelain);

        if self.selected >= self.files.len() {
            self.selected = self.files.len().saturating_sub(1);
        }

        self.log_lines = git_capture(&[
            "log",
            "--graph",
            "--all",
            "--decorate",
            "--date=short",
            "--pretty=format:%h %d %s [%an %ad]",
            "-n",
            "120",
            "--color=never",
        ])?
        .lines()
        .map(ToString::to_string)
        .collect();

        self.log_commits = self
            .log_lines
            .iter()
            .map(|l| parse_commit_hash(l))
            .collect();
        if self.log_selected >= self.log_lines.len() {
            self.log_selected = self.log_lines.len().saturating_sub(1);
        }

        self.refresh_diff();
        self.refresh_commit_diff();
        Ok(())
    }

    pub(super) fn refresh_diff(&mut self) {
        self.diff_lines.clear();
        if self.files.is_empty() {
            self.diff_lines.push("Working tree clean.".to_string());
            return;
        }

        let selected = &self.files[self.selected].git_path;
        let mut args = vec!["diff", "--no-color", "--", selected.as_str()];
        let mut diff = git_capture(&args).unwrap_or_else(|e| format!("diff error: {e}"));

        if diff.trim().is_empty() {
            args = vec!["diff", "--staged", "--no-color", "--", selected.as_str()];
            diff = git_capture(&args).unwrap_or_else(|e| format!("diff error: {e}"));
        }

        if diff.trim().is_empty() {
            self.diff_lines
                .push("No diff for selected file.".to_string());
            return;
        }

        self.diff_lines = diff.lines().map(ToString::to_string).collect();
    }

    pub(super) fn refresh_commit_diff(&mut self) {
        if self.log_lines.is_empty() {
            self.commit_diff_lines = vec!["No commits found.".to_string()];
            self.commit_diff_scroll = 0;
            return;
        }

        let mut idx = self.log_selected;
        let mut picked: Option<String> = None;
        while idx < self.log_commits.len() {
            if let Some(hash) = &self.log_commits[idx] {
                picked = Some(hash.clone());
                break;
            }
            idx += 1;
        }
        if picked.is_none() {
            idx = self.log_selected;
            while idx > 0 {
                idx -= 1;
                if let Some(hash) = &self.log_commits[idx] {
                    picked = Some(hash.clone());
                    break;
                }
            }
        }

        let Some(hash) = picked else {
            self.commit_diff_lines = vec!["No commit selected.".to_string()];
            self.commit_diff_scroll = 0;
            return;
        };

        let output = git_capture(&["show", "--stat", "--patch", "--color=never", &hash])
            .unwrap_or_else(|e| format!("commit diff error: {e}"));
        if output.trim().is_empty() {
            self.commit_diff_lines = vec![format!("No diff output for commit {hash}")];
        } else {
            self.commit_diff_lines = output.lines().map(ToString::to_string).collect();
        }
        self.commit_diff_scroll = cmp::min(
            self.commit_diff_scroll,
            self.commit_diff_lines.len().saturating_sub(1),
        );
    }

    pub(super) fn move_home_active(&mut self) {
        match self.tab {
            Tab::Workspace => self.selected = 0,
            Tab::Graph => self.log_selected = 0,
            Tab::CommitDiff => self.commit_diff_scroll = 0,
        }
    }

    pub(super) fn move_end_active(&mut self) {
        match self.tab {
            Tab::Workspace => self.selected = self.files.len().saturating_sub(1),
            Tab::Graph => self.log_selected = self.log_lines.len().saturating_sub(1),
            Tab::CommitDiff => {
                self.commit_diff_scroll = self.commit_diff_lines.len().saturating_sub(1)
            }
        }
    }

    pub(super) fn move_up_active(&mut self) {
        match self.tab {
            Tab::Workspace => self.move_up(),
            Tab::Graph => {
                self.log_selected = self.log_selected.saturating_sub(1);
                self.refresh_commit_diff();
            }
            Tab::CommitDiff => {
                self.commit_diff_scroll = self.commit_diff_scroll.saturating_sub(1);
            }
        }
    }

    pub(super) fn move_down_active(&mut self) {
        match self.tab {
            Tab::Workspace => self.move_down(),
            Tab::Graph => {
                self.log_selected = cmp::min(
                    self.log_selected + 1,
                    self.log_lines.len().saturating_sub(1),
                );
                self.refresh_commit_diff();
            }
            Tab::CommitDiff => {
                self.commit_diff_scroll = cmp::min(
                    self.commit_diff_scroll + 1,
                    self.commit_diff_lines.len().saturating_sub(1),
                );
            }
        }
    }

    pub(super) fn move_up(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }

    pub(super) fn move_down(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.selected = cmp::min(self.selected + 1, self.files.len().saturating_sub(1));
    }

    pub(super) fn stage_selected(&mut self) -> Result<(), String> {
        if self.files.is_empty() {
            self.status_msg = "No files to stage".to_string();
            return Ok(());
        }
        let path = self.files[self.selected].git_path.clone();
        git_status(&["add", "--", &path])?;
        self.status_msg = format!("Staged {path}");
        self.refresh()?;
        Ok(())
    }

    pub(super) fn unstage_selected(&mut self) -> Result<(), String> {
        if self.files.is_empty() {
            self.status_msg = "No files to unstage".to_string();
            return Ok(());
        }
        let path = self.files[self.selected].git_path.clone();
        git_status(&["restore", "--staged", "--", &path])?;
        self.status_msg = format!("Unstaged {path}");
        self.refresh()?;
        Ok(())
    }

    pub(super) fn stage_all(&mut self) -> Result<(), String> {
        git_status(&["add", "-A"])?;
        self.status_msg = "Staged all changes".to_string();
        self.refresh()?;
        Ok(())
    }

    pub(super) fn unstage_all(&mut self) -> Result<(), String> {
        git_status(&["restore", "--staged", "."])?;
        self.status_msg = "Unstaged all files".to_string();
        self.refresh()?;
        Ok(())
    }

    pub(super) fn push_current_branch(&mut self) -> Result<(), String> {
        let branch = if self.branch.trim().is_empty() {
            git_capture(&["branch", "--show-current"])?
                .trim()
                .to_string()
        } else {
            self.branch.trim().to_string()
        };

        if branch.is_empty() {
            self.status_msg = "Could not determine current branch".to_string();
            return Ok(());
        }

        git_status(&["push", "-u", "origin", &branch])?;
        self.status_msg = format!("Pushed branch {branch}");
        Ok(())
    }

    pub(super) fn push_to_remote_branch(
        &mut self,
        remote: &str,
        branch: &str,
        set_upstream: bool,
    ) -> Result<(), String> {
        if remote.trim().is_empty() || branch.trim().is_empty() {
            self.status_msg = "Usage: remote and branch are required".to_string();
            return Ok(());
        }
        let refspec = format!("HEAD:{branch}");
        if set_upstream {
            git_status(&["push", "-u", remote, &refspec])?;
        } else {
            git_status(&["push", remote, &refspec])?;
        }
        self.status_msg = format!("Pushed to {remote}/{branch}");
        self.refresh()?;
        Ok(())
    }

    pub(super) fn run_command(&mut self, cmdline: &str) -> Result<bool, String> {
        let parts = cmdline.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            self.status_msg = "No command entered".to_string();
            return Ok(false);
        }

        match parts[0] {
            "q" | "quit" | "exit" => return Ok(true),
            "help" => {
                self.overlay = Some(Overlay::Help);
                self.status_msg = "Help opened".to_string();
            }
            "cmdhelp" | "commands" => {
                self.status_msg = command_mode_help();
            }
            "refresh" | "r" => {
                self.refresh()?;
                self.status_msg = "Refreshed".to_string();
            }
            "stage" => self.stage_selected()?,
            "unstage" => self.unstage_selected()?,
            "stage-all" => self.stage_all()?,
            "unstage-all" => self.unstage_all()?,
            "commit" => {
                let msg = cmdline.trim_start_matches("commit").trim();
                if msg.is_empty() {
                    self.status_msg = "Usage: commit <message>".to_string();
                } else {
                    git_status(&["commit", "-m", msg])?;
                    self.status_msg = "Commit created".to_string();
                    self.refresh()?;
                }
            }
            "push" => self.push_current_branch()?,
            "push-remote" | "pushremote" => {
                if parts.len() == 1 {
                    self.input_mode = InputMode::PushRemote;
                    self.input = if let Some(up) = &self.upstream {
                        up.replace('/', " ")
                    } else {
                        format!("origin {}", self.branch)
                    };
                    self.status_msg = "Enter remote and branch, then press Enter".to_string();
                } else if parts.len() == 2 {
                    if let Some((remote, branch)) = parts[1].split_once('/') {
                        self.push_to_remote_branch(remote, branch, true)?;
                    } else {
                        self.status_msg =
                            "Usage: push-remote <remote>/<branch> or <remote> <branch>".to_string();
                    }
                } else {
                    self.push_to_remote_branch(parts[1], parts[2], true)?;
                }
            }
            "branch" => {
                if parts.len() < 2 {
                    self.status_msg = "Usage: branch <name>".to_string();
                } else {
                    git_status(&["switch", "-c", parts[1]])?;
                    self.status_msg = format!("Created and switched to {}", parts[1]);
                    self.refresh()?;
                }
            }
            "switch" => {
                if parts.len() < 2 {
                    self.status_msg = "Usage: switch <name>".to_string();
                } else {
                    git_status(&["switch", parts[1]])?;
                    self.status_msg = format!("Switched to {}", parts[1]);
                    self.refresh()?;
                }
            }
            "log" => {
                self.pane = Pane::Log;
                self.tab = Tab::Workspace;
                self.status_msg = "Focused log pane".to_string();
            }
            "diff" => {
                self.pane = Pane::Diff;
                self.tab = Tab::Workspace;
                self.status_msg = "Focused diff pane".to_string();
            }
            "status" => {
                self.pane = Pane::Files;
                self.tab = Tab::Workspace;
                self.status_msg = "Focused status pane".to_string();
            }
            "workspace" => {
                self.tab = Tab::Workspace;
                self.status_msg = "Switched to Workspace tab".to_string();
            }
            "graph-tab" | "graphview" => {
                self.tab = Tab::Graph;
                self.status_msg = "Switched to Graph tab".to_string();
            }
            "commitdiff" | "commit-diff" => {
                self.tab = Tab::CommitDiff;
                self.status_msg = "Switched to CommitDiff tab".to_string();
            }
            "themes" => {
                self.status_msg = format!(
                    "Themes: {}",
                    THEMES.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
                );
            }
            "theme" => {
                if parts.len() < 2 {
                    self.status_msg = "Usage: theme <name>".to_string();
                } else if self.apply_theme(parts[1]) {
                    self.status_msg = format!("Theme switched to {}", self.theme.name);
                } else {
                    self.status_msg = format!("Unknown theme: {}", parts[1]);
                }
            }
            "palette" => self.open_palette(),
            _ => {
                self.status_msg = format!("Unknown command: {}", parts[0]);
            }
        }

        self.refresh_diff();
        self.refresh_commit_diff();
        Ok(false)
    }

    pub(super) fn open_palette(&mut self) {
        self.overlay = Some(Overlay::Palette);
        self.palette_query.clear();
        self.palette_selected = 0;
        self.status_msg = "Command palette opened".to_string();
    }

    pub(super) fn cycle_theme(&mut self) {
        let idx = THEMES
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = THEMES[(idx + 1) % THEMES.len()];
        self.status_msg = format!("Theme: {}", self.theme.name);
    }

    pub(super) fn apply_theme(&mut self, name: &str) -> bool {
        if let Some(found) = THEMES.iter().find(|t| t.name.eq_ignore_ascii_case(name)) {
            self.theme = *found;
            true
        } else {
            false
        }
    }

    pub(super) fn palette_entries(&self) -> Vec<PaletteEntry> {
        let mut entries = vec![
            PaletteEntry {
                label: "Refresh".to_string(),
                action: PaletteAction::Command("refresh"),
            },
            PaletteEntry {
                label: "Stage Selected".to_string(),
                action: PaletteAction::Command("stage"),
            },
            PaletteEntry {
                label: "Unstage Selected".to_string(),
                action: PaletteAction::Command("unstage"),
            },
            PaletteEntry {
                label: "Stage All".to_string(),
                action: PaletteAction::Command("stage-all"),
            },
            PaletteEntry {
                label: "Unstage All".to_string(),
                action: PaletteAction::Command("unstage-all"),
            },
            PaletteEntry {
                label: "Push Current Branch".to_string(),
                action: PaletteAction::Command("push"),
            },
            PaletteEntry {
                label: "Push To Remote Branch".to_string(),
                action: PaletteAction::Command("push-remote"),
            },
            PaletteEntry {
                label: "Open Workspace Tab".to_string(),
                action: PaletteAction::Command("workspace"),
            },
            PaletteEntry {
                label: "Open Graph Tab".to_string(),
                action: PaletteAction::Command("graph-tab"),
            },
            PaletteEntry {
                label: "Open CommitDiff Tab".to_string(),
                action: PaletteAction::Command("commitdiff"),
            },
            PaletteEntry {
                label: "Show Help".to_string(),
                action: PaletteAction::Command("help"),
            },
            PaletteEntry {
                label: "Command Mode Help".to_string(),
                action: PaletteAction::Command("cmdhelp"),
            },
            PaletteEntry {
                label: "Quit".to_string(),
                action: PaletteAction::Command("quit"),
            },
        ];

        for t in THEMES {
            entries.push(PaletteEntry {
                label: format!("Theme: {}", t.name),
                action: PaletteAction::Theme(t.name),
            });
        }

        entries
    }

    pub(super) fn filtered_palette_entries(&self) -> Vec<PaletteEntry> {
        let q = self.palette_query.trim().to_lowercase();
        if q.is_empty() {
            return self.palette_entries();
        }
        self.palette_entries()
            .into_iter()
            .filter(|e| e.label.to_lowercase().contains(&q))
            .collect()
    }

    pub(super) fn clamp_palette_selected(&mut self) {
        let len = self.filtered_palette_entries().len();
        if len == 0 {
            self.palette_selected = 0;
        } else {
            self.palette_selected = cmp::min(self.palette_selected, len - 1);
        }
    }

    pub(super) fn select_next_palette(&mut self) {
        let len = self.filtered_palette_entries().len();
        if len == 0 {
            self.palette_selected = 0;
            return;
        }
        self.palette_selected = (self.palette_selected + 1) % len;
    }

    pub(super) fn select_prev_palette(&mut self) {
        let len = self.filtered_palette_entries().len();
        if len == 0 {
            self.palette_selected = 0;
            return;
        }
        if self.palette_selected == 0 {
            self.palette_selected = len - 1;
        } else {
            self.palette_selected -= 1;
        }
    }

    pub(super) fn execute_palette_selection(&mut self) -> Result<bool, String> {
        let entries = self.filtered_palette_entries();
        if entries.is_empty() {
            self.status_msg = "No palette matches".to_string();
            return Ok(false);
        }

        let idx = cmp::min(self.palette_selected, entries.len() - 1);
        let chosen = entries[idx].clone();
        self.overlay = None;

        match chosen.action {
            PaletteAction::Command(cmd) => self.run_command(cmd),
            PaletteAction::Theme(name) => {
                if self.apply_theme(name) {
                    self.status_msg = format!("Theme switched to {}", name);
                } else {
                    self.status_msg = format!("Unknown theme: {}", name);
                }
                Ok(false)
            }
        }
    }

    pub(super) fn status_rows(&self) -> Vec<StatusRow> {
        let mut tracked = Vec::new();
        let mut untracked = Vec::new();
        let mut other = Vec::new();

        for (idx, file) in self.files.iter().enumerate() {
            if file.is_untracked() {
                untracked.push(idx);
            } else if file.is_tracked_change() {
                tracked.push(idx);
            } else {
                other.push(idx);
            }
        }

        let mut rows = Vec::new();
        if !tracked.is_empty() {
            rows.push(StatusRow::Header(" TRACKED (modified/deleted) "));
            rows.extend(tracked.into_iter().map(StatusRow::File));
            rows.push(StatusRow::Spacer);
        }
        if !untracked.is_empty() {
            rows.push(StatusRow::Header(" UNTRACKED "));
            rows.extend(untracked.into_iter().map(StatusRow::File));
            rows.push(StatusRow::Spacer);
        }
        if !other.is_empty() {
            rows.push(StatusRow::Header(" OTHER CHANGES "));
            rows.extend(other.into_iter().map(StatusRow::File));
        }
        if matches!(rows.last(), Some(StatusRow::Spacer)) {
            rows.pop();
        }
        rows
    }

    pub(super) fn scroll_active_up(&mut self) {
        if self.overlay == Some(Overlay::Palette) {
            self.select_prev_palette();
            return;
        }
        self.move_up_active();
    }

    pub(super) fn scroll_active_down(&mut self) {
        if self.overlay == Some(Overlay::Palette) {
            self.select_next_palette();
            return;
        }
        self.move_down_active();
    }
}

pub(super) fn parse_commit_hash(line: &str) -> Option<String> {
    line.split_whitespace().find_map(|tok| {
        if tok.len() >= 7 && tok.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(tok.to_string())
        } else {
            None
        }
    })
}

pub(super) fn command_mode_help() -> String {
    "Cmds: help|cmdhelp|refresh|stage|unstage|stage-all|unstage-all|commit <msg>|push|push-remote <remote>/<branch>|branch <name>|switch <name>|workspace|graph-tab|commitdiff|theme <name>|themes|palette|quit".to_string()
}

fn parse_porcelain(s: &str) -> Vec<FileStatus> {
    let mut out = Vec::new();
    for line in s.lines() {
        if line.len() < 4 {
            continue;
        }

        let x = line.chars().next().unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        let mut display = line[3..].to_string();
        let mut git_path = display.clone();

        if let Some((_, new)) = display.rsplit_once(" -> ") {
            git_path = new.to_string();
            display = format!("{} => {}", display.split(" -> ").next().unwrap_or(""), new);
        }

        out.push(FileStatus {
            x,
            y,
            display_path: display,
            git_path,
        });
    }
    out
}

pub(super) fn git_capture(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git {} failed: {e}", args.join(" ")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub(super) fn git_status(args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git {} failed: {e}", args.join(" ")))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
