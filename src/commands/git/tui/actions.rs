use std::cmp;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::process;

use super::*;

impl App {
    pub(super) fn new(theme: Theme, style: Style) -> Self {
        Self {
            theme,
            style,
            pane: Pane::Files,
            diff_mode: DiffMode::SelectedFile,
            detail_diff_mode: DetailDiffMode::Commit,
            detail_pane: DetailPane::Diff,
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
            selected_commit: None,
            workspace_diff: DiffView::default(),
            detail_diff: DiffView::default(),
            push_overlay_lines: Vec::new(),
            push_overlay_ok: None,
            action_overlay_title: "Push".to_string(),
            branch_choices: Vec::new(),
            branch_pick_selected: 0,
            conflict_target: None,
            conflict_pick_selected: 0,
            conflict_selected: 0,
            conflict_marked: std::collections::HashSet::new(),
            stashes: Vec::new(),
            stash_selected: 0,
            history_choices: Vec::new(),
            reset_selected: 0,
            reset_hard: false,
            squash_marked: std::collections::HashSet::new(),
            pending_action: None,
        }
    }

    pub(super) fn needs_cursor_blink(&self) -> bool {
        self.input_mode != InputMode::None || self.overlay == Some(Overlay::Palette)
    }

    pub(super) fn refresh(&mut self) -> Result<(), String> {
        let selected_path = self.files.get(self.selected).map(|file| file.git_path.clone());
        self.branch = git_capture(&["branch", "--show-current"])?
            .trim()
            .to_string();
        self.upstream = git_capture(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let porcelain = git_capture(&["status", "--porcelain=1"])?;
        self.files = parse_porcelain(&porcelain);

        if let Some(path) = selected_path {
            if let Some(idx) = self.files.iter().position(|file| file.git_path == path) {
                self.selected = idx;
            }
        }
        if self.selected >= self.files.len() {
            self.selected = self.files.len().saturating_sub(1);
        }
        let conflicts = self.conflict_paths();
        if self.conflict_selected >= conflicts.len() {
            self.conflict_selected = conflicts.len().saturating_sub(1);
        }
        self.conflict_marked
            .retain(|p| conflicts.iter().any(|q| q == p));

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
        self.refresh_detail_diff();
        self.refresh_stashes()?;
        Ok(())
    }

    fn refresh_stashes(&mut self) -> Result<(), String> {
        let output = git_capture(&["stash", "list", "--format=%gd%x1f%gs%x1f%cr"])?;
        self.stashes = output
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\x1f');
                Some(StashEntry {
                    reference: fields.next()?.to_string(),
                    message: fields.next().unwrap_or_default().to_string(),
                    age: fields.next().unwrap_or_default().to_string(),
                })
            })
            .collect();
        if self.stash_selected >= self.stashes.len() {
            self.stash_selected = self.stashes.len().saturating_sub(1);
        }
        Ok(())
    }

    pub(super) fn create_stash(&mut self, message: &str) -> Result<(), String> {
        let mut args = vec!["stash", "push", "--include-untracked"];
        if !message.trim().is_empty() {
            args.extend(["-m", message.trim()]);
        }
        git_status(&args)?;
        self.status_msg = "Stashed working changes".to_string();
        self.refresh()
    }

    pub(super) fn apply_selected_stash(&mut self, pop: bool) -> Result<(), String> {
        let Some(stash) = self.stashes.get(self.stash_selected).cloned() else {
            self.status_msg = "No stash selected".to_string();
            return Ok(());
        };
        git_status(&[if pop { "stash" } else { "stash" }, if pop { "pop" } else { "apply" }, &stash.reference])?;
        self.status_msg = format!("{} {}", if pop { "Restored" } else { "Applied" }, stash.reference);
        self.refresh()
    }

    pub(super) fn add_selected_to_gitignore(&mut self) -> Result<(), String> {
        let Some(path) = self.files.get(self.selected).map(|file| file.git_path.clone()) else {
            self.status_msg = "No changed file selected".to_string();
            return Ok(());
        };
        self.add_to_gitignore(&path)
    }

    pub(super) fn add_to_gitignore(&mut self, raw: &str) -> Result<(), String> {
        let entry = normalize_gitignore_entry(raw)?;
        let root = git_capture(&["rev-parse", "--show-toplevel"])?;
        let gitignore = PathBuf::from(root.trim()).join(".gitignore");
        let existing = fs::read_to_string(&gitignore).unwrap_or_default();
        if existing.lines().map(str::trim).any(|line| line == entry) {
            self.status_msg = format!("Already ignored: {entry}");
            return Ok(());
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gitignore)
            .map_err(|e| format!("failed opening {}: {e}", gitignore.display()))?;
        if !existing.is_empty() && !existing.ends_with('\n') {
            writeln!(file).map_err(|e| format!("failed updating .gitignore: {e}"))?;
        }
        writeln!(file, "{entry}").map_err(|e| format!("failed updating .gitignore: {e}"))?;
        self.status_msg = format!("Added to .gitignore: {entry}");
        self.refresh()
    }

    pub(super) fn open_reset_picker(&mut self) -> Result<(), String> {
        self.history_choices = self.branch_history()?;
        self.reset_selected = 0;
        self.reset_hard = false;
        self.overlay = Some(Overlay::ResetPicker);
        Ok(())
    }

    pub(super) fn open_squash_picker(&mut self) -> Result<(), String> {
        self.history_choices = self.branch_history()?;
        self.squash_marked.clear();
        self.overlay = Some(Overlay::SquashPicker);
        Ok(())
    }

    fn branch_history(&self) -> Result<Vec<HistoryEntry>, String> {
        let output = git_capture(&["log", "--first-parent", "--format=%H%x1f%s", "-n", "60"])?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let (hash, subject) = line.split_once('\x1f')?;
                Some(HistoryEntry {
                    hash: hash.to_string(),
                    subject: subject.to_string(),
                })
            })
            .collect())
    }

    pub(super) fn select_next_history(&mut self) {
        self.reset_selected = next_index(self.reset_selected, self.history_choices.len());
    }

    pub(super) fn select_prev_history(&mut self) {
        self.reset_selected = prev_index(self.reset_selected, self.history_choices.len());
    }

    pub(super) fn toggle_squash_mark(&mut self) {
        let Some(entry) = self.history_choices.get(self.reset_selected) else {
            return;
        };
        if !self.squash_marked.insert(entry.hash.clone()) {
            self.squash_marked.remove(&entry.hash);
        }
    }

    pub(super) fn prepare_reset(&mut self) {
        let Some(entry) = self.history_choices.get(self.reset_selected) else {
            self.status_msg = "No commit selected".to_string();
            return;
        };
        self.pending_action = Some(PendingAction::Reset {
            target: entry.hash.clone(),
            hard: self.reset_hard,
        });
        self.overlay = Some(Overlay::Confirm);
    }

    pub(super) fn prepare_squash(&mut self, message: &str) -> Result<(), String> {
        if message.trim().is_empty() {
            self.status_msg = "Squash commit message cannot be empty".to_string();
            return Ok(());
        }
        if self.squash_marked.len() < 2 {
            self.status_msg = "Mark at least two commits to squash".to_string();
            return Ok(());
        }
        let marked = self
            .history_choices
            .iter()
            .enumerate()
            .filter(|(_, entry)| self.squash_marked.contains(&entry.hash))
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();
        let Some(&oldest_idx) = marked.iter().max() else {
            return Ok(());
        };
        let Some(&newest_idx) = marked.iter().min() else {
            return Ok(());
        };
        if newest_idx != 0 || marked.len() != oldest_idx + 1 {
            self.status_msg = "Squash selection must be a contiguous range ending at HEAD".to_string();
            return Ok(());
        }
        let oldest = self.history_choices[oldest_idx].hash.clone();
        self.pending_action = Some(PendingAction::Squash {
            oldest,
            message: message.trim().to_string(),
        });
        self.overlay = Some(Overlay::Confirm);
        Ok(())
    }

    pub(super) fn execute_pending_action(&mut self) -> Result<(), String> {
        let Some(action) = self.pending_action.take() else {
            self.overlay = None;
            return Ok(());
        };
        self.overlay = None;
        match action {
            PendingAction::Reset { target, hard } => {
                git_status(&["reset", if hard { "--hard" } else { "--soft" }, &target])?;
                self.status_msg = format!("{} reset to {}", if hard { "Hard" } else { "Soft" }, short_hash(&target));
            }
            PendingAction::Squash { oldest, message } => {
                let parent = format!("{oldest}^");
                git_status(&["reset", "--soft", &parent])?;
                git_status(&["commit", "-m", &message])?;
                self.status_msg = format!("Squashed commits into {}", short_hash(&oldest));
            }
        }
        self.squash_marked.clear();
        self.refresh()
    }

    pub(super) fn refresh_diff(&mut self) {
        match self.diff_mode {
            DiffMode::SelectedFile => {
                if self.files.is_empty() {
                    self.workspace_diff
                        .replace_lines(vec!["Working tree clean.".to_string()]);
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
                    self.workspace_diff
                        .replace_lines(vec!["No diff for selected file.".to_string()]);
                    return;
                }

                self.workspace_diff
                    .replace_lines(diff.lines().map(ToString::to_string).collect());
            }
            DiffMode::Repo => {
                let diff = git_capture(&["diff", "--no-color", "--unified=3"])
                    .unwrap_or_else(|e| format!("diff error: {e}"));
                if diff.trim().is_empty() {
                    self.workspace_diff
                        .replace_lines(vec!["No repo diff.".to_string()]);
                    return;
                }
                self.workspace_diff
                    .replace_lines(diff.lines().map(ToString::to_string).collect());
            }
        }
    }

    pub(super) fn toggle_diff_mode(&mut self) {
        self.diff_mode = match self.diff_mode {
            DiffMode::SelectedFile => DiffMode::Repo,
            DiffMode::Repo => DiffMode::SelectedFile,
        };
        self.refresh_diff();
        self.status_msg = match self.diff_mode {
            DiffMode::SelectedFile => "Diff mode: selected file".to_string(),
            DiffMode::Repo => "Diff mode: full repo".to_string(),
        };
    }

    pub(super) fn toggle_detail_diff_mode(&mut self) {
        self.detail_diff_mode = match self.detail_diff_mode {
            DetailDiffMode::Commit => DetailDiffMode::Repo,
            DetailDiffMode::Repo => DetailDiffMode::SelectedFile,
            DetailDiffMode::SelectedFile => DetailDiffMode::Commit,
        };
        self.detail_pane = if self.detail_diff_mode == DetailDiffMode::SelectedFile {
            DetailPane::Files
        } else {
            DetailPane::Diff
        };
        self.refresh_detail_diff();
        self.status_msg = match self.detail_diff_mode {
            DetailDiffMode::Commit => "Detail diff: selected commit".to_string(),
            DetailDiffMode::Repo => "Detail diff: repository".to_string(),
            DetailDiffMode::SelectedFile => "Detail diff: selected file".to_string(),
        };
    }

    pub(super) fn refresh_detail_diff(&mut self) {
        self.selected_commit = None;
        match self.detail_diff_mode {
            DetailDiffMode::SelectedFile => {
                if self.files.is_empty() {
                    self.detail_diff
                        .replace_lines(vec!["Working tree clean.".to_string()]);
                    return;
                }
                let path = self.files[self.selected].git_path.clone();
                let mut output = git_capture(&["diff", "--no-color", "--", &path])
                    .unwrap_or_else(|e| format!("diff error: {e}"));
                if output.trim().is_empty() {
                    output = git_capture(&["diff", "--staged", "--no-color", "--", &path])
                        .unwrap_or_else(|e| format!("diff error: {e}"));
                }
                self.detail_diff.replace_lines(if output.trim().is_empty() {
                    vec!["No diff for selected file.".to_string()]
                } else {
                    output.lines().map(ToString::to_string).collect()
                });
                return;
            }
            DetailDiffMode::Repo => {
                let output = git_capture(&["diff", "--no-color", "--unified=3"])
                    .unwrap_or_else(|e| format!("diff error: {e}"));
                self.detail_diff.replace_lines(if output.trim().is_empty() {
                    vec!["No repo diff.".to_string()]
                } else {
                    output.lines().map(ToString::to_string).collect()
                });
                return;
            }
            DetailDiffMode::Commit => {}
        }

        if self.log_lines.is_empty() {
            self.detail_diff.replace_lines(vec!["No commits found.".to_string()]);
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
            self.detail_diff
                .replace_lines(vec!["No commit selected.".to_string()]);
            return;
        };
        self.selected_commit = Some(hash.clone());

        let output = git_capture(&["show", "--stat", "--patch", "--color=never", &hash])
            .unwrap_or_else(|e| format!("commit diff error: {e}"));
        if output.trim().is_empty() {
            self.detail_diff
                .replace_lines(vec![format!("No diff output for commit {hash}")]);
        } else {
            self.detail_diff
                .replace_lines(output.lines().map(ToString::to_string).collect());
        }
    }

    pub(super) fn move_home_active(&mut self) {
        match self.tab {
            Tab::Workspace => match self.pane {
                Pane::Files => self.selected = 0,
                Pane::Log => self.log_selected = 0,
                Pane::Diff => self.workspace_diff.scroll = 0,
            },
            Tab::Graph => self.log_selected = 0,
            Tab::CommitDiff => match self.detail_pane {
                DetailPane::Files => self.selected = self.file_selection_order().first().copied().unwrap_or(0),
                DetailPane::Diff => self.detail_diff.scroll = 0,
            },
            Tab::Conflicts => self.conflict_selected = 0,
            Tab::Stashes => self.stash_selected = 0,
        }
    }

    pub(super) fn move_end_active(&mut self) {
        match self.tab {
            Tab::Workspace => match self.pane {
                Pane::Files => self.selected = self.file_selection_order().last().copied().unwrap_or(0),
                Pane::Log => self.log_selected = self.log_lines.len().saturating_sub(1),
                Pane::Diff => self.workspace_diff.scroll = self.workspace_diff_max_scroll(),
            },
            Tab::Graph => self.log_selected = self.log_lines.len().saturating_sub(1),
            Tab::CommitDiff => match self.detail_pane {
                DetailPane::Files => self.selected = self.file_selection_order().last().copied().unwrap_or(0),
                DetailPane::Diff => self.detail_diff.scroll = self.detail_diff_max_scroll(),
            },
            Tab::Conflicts => {
                self.conflict_selected = self.conflict_paths().len().saturating_sub(1)
            }
            Tab::Stashes => self.stash_selected = self.stashes.len().saturating_sub(1),
        }
    }

    pub(super) fn move_up_active(&mut self) {
        match self.tab {
            Tab::Workspace => match self.pane {
                Pane::Files => self.move_up(),
                Pane::Log => self.log_selected = prev_index(self.log_selected, self.log_lines.len()),
                Pane::Diff => self.workspace_diff.scroll = self.workspace_diff.scroll.saturating_sub(1),
            },
            Tab::Graph => {
                self.log_selected = prev_index(self.log_selected, self.log_lines.len());
                self.refresh_detail_diff();
            }
            Tab::CommitDiff => {
                match self.detail_pane {
                    DetailPane::Files => self.move_up(),
                    DetailPane::Diff => self.detail_diff.scroll = self.detail_diff.scroll.saturating_sub(1),
                }
            }
            Tab::Conflicts => {
                self.conflict_selected = prev_index(self.conflict_selected, self.conflict_paths().len());
            }
            Tab::Stashes => self.stash_selected = prev_index(self.stash_selected, self.stashes.len()),
        }
    }

    pub(super) fn move_down_active(&mut self) {
        match self.tab {
            Tab::Workspace => match self.pane {
                Pane::Files => self.move_down(),
                Pane::Log => self.log_selected = next_index(self.log_selected, self.log_lines.len()),
                Pane::Diff => {
                    self.workspace_diff.scroll = cmp::min(
                        self.workspace_diff.scroll + 1,
                        self.workspace_diff_max_scroll(),
                    );
                }
            },
            Tab::Graph => {
                self.log_selected = next_index(self.log_selected, self.log_lines.len());
                self.refresh_detail_diff();
            }
            Tab::CommitDiff => {
                match self.detail_pane {
                    DetailPane::Files => self.move_down(),
                    DetailPane::Diff => {
                        self.detail_diff.scroll = cmp::min(
                            self.detail_diff.scroll + 1,
                            self.detail_diff_max_scroll(),
                        );
                    }
                }
            }
            Tab::Conflicts => {
                self.conflict_selected = next_index(self.conflict_selected, self.conflict_paths().len());
            }
            Tab::Stashes => self.stash_selected = next_index(self.stash_selected, self.stashes.len()),
        }
    }

    pub(super) fn move_up(&mut self) {
        let order = self.file_selection_order();
        if order.is_empty() {
            return;
        }
        let before = self.selected;
        let at = order.iter().position(|idx| *idx == self.selected).unwrap_or(0);
        self.selected = order[prev_index(at, order.len())];
        if self.selected != before {
            self.refresh_diff();
            self.refresh_detail_diff();
        }
    }

    pub(super) fn move_down(&mut self) {
        let order = self.file_selection_order();
        if order.is_empty() {
            return;
        }
        let before = self.selected;
        let at = order.iter().position(|idx| *idx == self.selected).unwrap_or(0);
        self.selected = order[next_index(at, order.len())];
        if self.selected != before {
            self.refresh_diff();
            self.refresh_detail_diff();
        }
    }

    pub(super) fn file_selection_order(&self) -> Vec<usize> {
        self.status_rows()
            .into_iter()
            .filter_map(|row| match row {
                StatusRow::File(idx) => Some(idx),
                StatusRow::Header(_) | StatusRow::Spacer => None,
            })
            .collect()
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

        let refspec = format!("HEAD:{branch}");
        self.run_git_with_overlay(
            "Push".to_string(),
            format!("Pushing current branch to origin/{branch}"),
            vec![
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                refspec,
            ],
        )?;
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
        let mut args = vec!["push".to_string()];
        if set_upstream {
            args.push("-u".to_string());
        }
        args.push(remote.to_string());
        args.push(refspec);

        self.run_git_with_overlay(
            "Push".to_string(),
            format!("Pushing to {remote}/{branch}"),
            args,
        )?;
        Ok(())
    }

    pub(super) fn fetch_all(&mut self) -> Result<(), String> {
        self.run_git_with_overlay(
            "Fetch".to_string(),
            "Fetching all remotes (prune enabled)".to_string(),
            vec![
                "fetch".to_string(),
                "--all".to_string(),
                "--prune".to_string(),
            ],
        )
    }

    pub(super) fn pull_current_branch(&mut self) -> Result<(), String> {
        self.run_git_with_overlay(
            "Pull".to_string(),
            format!("Pulling current branch {}", self.branch),
            vec!["pull".to_string(), "--rebase".to_string()],
        )
    }

    fn run_git_with_overlay(
        &mut self,
        overlay_title: String,
        what: String,
        args: Vec<String>,
    ) -> Result<(), String> {
        self.overlay = Some(Overlay::Push);
        self.action_overlay_title = overlay_title.clone();
        self.push_overlay_ok = None;
        self.push_overlay_lines = vec![
            what.clone(),
            format!("$ git {}", args.join(" ")),
            format!("Running {overlay_title}... input is blocked until completion."),
        ];
        self.render(true)?;

        let mut child = Command::new("git")
            .args(args.iter().map(String::as_str))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("git push failed to start: {e}"))?;

        let frames = [
            "██████░░░░░░░░░░░░░░",
            "░██████░░░░░░░░░░░░░",
            "░░██████░░░░░░░░░░░░",
            "░░░██████░░░░░░░░░░░",
            "░░░░██████░░░░░░░░░░",
            "░░░░░██████░░░░░░░░░",
            "░░░░░░██████░░░░░░░░",
            "░░░░░░░██████░░░░░░░",
            "░░░░░░░░██████░░░░░░",
            "░░░░░░░░░██████░░░░░",
            "░░░░░░░░░░██████░░░░",
            "░░░░░░░░░░░██████░░░",
            "░░░░░░░░░░░░██████░░",
            "░░░░░░░░░░░░░██████░",
            "░░░░░░░░░░░░░░██████",
            "░░░░░░░░░░░░░██████░",
            "░░░░░░░░░░░░██████░░",
            "░░░░░░░░░░░██████░░░",
            "░░░░░░░░░░██████░░░░",
            "░░░░░░░░░██████░░░░░",
            "░░░░░░░░██████░░░░░░",
            "░░░░░░░██████░░░░░░░",
            "░░░░░░██████░░░░░░░░",
            "░░░░░██████░░░░░░░░░",
            "░░░░██████░░░░░░░░░░",
            "░░░██████░░░░░░░░░░░",
            "░░██████░░░░░░░░░░░░",
            "░██████░░░░░░░░░░░░░",
        ];
        let start = Instant::now();
        let mut frame_idx = 0usize;
        loop {
            if child
                .try_wait()
                .map_err(|e| format!("git push failed during wait: {e}"))?
                .is_some()
            {
                break;
            }

            let elapsed = start.elapsed().as_secs_f32();
            self.push_overlay_lines = vec![
                what.clone(),
                format!("$ git {}", args.join(" ")),
                format!("Progress: [{}]", frames[frame_idx % frames.len()]),
                format!("Elapsed: {:.1}s", elapsed),
                format!("Running {overlay_title}... input is blocked until completion."),
            ];
            self.render(true)?;
            frame_idx = frame_idx.wrapping_add(1);
            thread::sleep(Duration::from_millis(70));
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("git push failed to collect output: {e}"))?;

        let ok = output.status.success();
        self.push_overlay_ok = Some(ok);
        self.push_overlay_lines = vec![if ok {
            format!("{overlay_title} completed successfully.")
        } else {
            format!("{overlay_title} failed.")
        }];
        self.push_overlay_lines
            .push("Progress: [████████████████████]".to_string());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut details = stdout
            .lines()
            .chain(stderr.lines())
            .filter(|l| !l.trim().is_empty())
            .take(14)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if details.is_empty() {
            details.push("No additional output from git push.".to_string());
        }
        self.push_overlay_lines.extend(details);
        self.push_overlay_lines
            .push("Press Enter or Esc to close.".to_string());

        if ok {
            self.status_msg = format!("{overlay_title} completed");
            let _ = self.refresh();
        } else {
            self.status_msg = format!("{overlay_title} failed (see overlay)");
        }

        self.render(true)?;
        Ok(())
    }

    pub(super) fn open_branch_picker(&mut self) -> Result<(), String> {
        let out = git_capture(&["branch", "--format=%(refname:short)"])?;
        let mut branches = out
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        branches.sort();
        branches.dedup();
        branches.insert(0, "<create-new-branch>".to_string());
        self.branch_choices = branches;
        self.branch_pick_selected = self
            .branch_choices
            .iter()
            .position(|b| b == &self.branch)
            .unwrap_or(0);
        self.overlay = Some(Overlay::BranchPicker);
        self.status_msg = "Branch picker opened".to_string();
        Ok(())
    }

    pub(super) fn select_next_branch_pick(&mut self) {
        if self.branch_choices.is_empty() {
            self.branch_pick_selected = 0;
            return;
        }
        self.branch_pick_selected = (self.branch_pick_selected + 1) % self.branch_choices.len();
    }

    pub(super) fn select_prev_branch_pick(&mut self) {
        if self.branch_choices.is_empty() {
            self.branch_pick_selected = 0;
            return;
        }
        if self.branch_pick_selected == 0 {
            self.branch_pick_selected = self.branch_choices.len() - 1;
        } else {
            self.branch_pick_selected -= 1;
        }
    }

    pub(super) fn run_selected_branch_pick(&mut self) -> Result<(), String> {
        if self.branch_choices.is_empty() {
            self.status_msg = "No branches available".to_string();
            self.overlay = None;
            return Ok(());
        }
        let choice = self.branch_choices[self.branch_pick_selected].clone();
        self.overlay = None;
        if choice == "<create-new-branch>" {
            self.input_mode = InputMode::NewBranch;
            self.input.clear();
            self.status_msg = "Enter new branch name".to_string();
            return Ok(());
        }
        git_status(&["switch", &choice])?;
        self.status_msg = format!("Switched to {choice}");
        self.refresh()?;
        Ok(())
    }

    pub(super) fn open_conflict_resolver(&mut self) {
        if self.files.is_empty() {
            self.status_msg = "No files in status".to_string();
            return;
        }
        let selected = &self.files[self.selected];
        if !selected.is_conflict() {
            self.status_msg = "Selected file is not in a merge conflict".to_string();
            return;
        }
        self.conflict_target = Some(selected.git_path.clone());
        self.conflict_pick_selected = 0;
        self.overlay = Some(Overlay::ConflictResolver);
    }

    pub(super) fn open_conflicts_tab(&mut self) {
        self.tab = Tab::Conflicts;
        if self.conflict_paths().is_empty() {
            self.status_msg = "No merge conflicts".to_string();
        } else {
            self.status_msg = "Conflicts tab opened".to_string();
        }
    }

    pub(super) fn conflict_paths(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| f.is_conflict())
            .map(|f| f.git_path.clone())
            .collect()
    }

    fn current_conflict_path(&self) -> Option<String> {
        let conflicts = self.conflict_paths();
        conflicts.get(self.conflict_selected).cloned()
    }

    pub(super) fn toggle_mark_conflict(&mut self) {
        let Some(path) = self.current_conflict_path() else {
            self.status_msg = "No conflict selected".to_string();
            return;
        };
        if !self.conflict_marked.insert(path.clone()) {
            self.conflict_marked.remove(&path);
        }
        self.status_msg = format!("Marked: {}", self.conflict_marked.len());
    }

    pub(super) fn mark_all_conflicts(&mut self) {
        let conflicts = self.conflict_paths();
        if conflicts.is_empty() {
            self.status_msg = "No conflicts to mark".to_string();
            return;
        }
        if self.conflict_marked.len() == conflicts.len() {
            self.conflict_marked.clear();
            self.status_msg = "Cleared conflict marks".to_string();
            return;
        }
        self.conflict_marked = conflicts.into_iter().collect();
        self.status_msg = format!("Marked {} conflicts", self.conflict_marked.len());
    }

    fn conflict_targets_for_apply(&self) -> Vec<String> {
        if !self.conflict_marked.is_empty() {
            let mut out = self
                .conflict_paths()
                .into_iter()
                .filter(|p| self.conflict_marked.contains(p))
                .collect::<Vec<_>>();
            out.sort();
            out.dedup();
            return out;
        }
        self.current_conflict_path().into_iter().collect()
    }

    pub(super) fn resolve_conflicts_ours_selected_or_marked(&mut self) -> Result<(), String> {
        let targets = self.conflict_targets_for_apply();
        self.resolve_conflicts_with_strategy(&targets, true)
    }

    pub(super) fn resolve_conflicts_theirs_selected_or_marked(&mut self) -> Result<(), String> {
        let targets = self.conflict_targets_for_apply();
        self.resolve_conflicts_with_strategy(&targets, false)
    }

    pub(super) fn resolve_conflicts_all_ours(&mut self) -> Result<(), String> {
        let targets = self.conflict_paths();
        self.resolve_conflicts_with_strategy(&targets, true)
    }

    pub(super) fn resolve_conflicts_all_theirs(&mut self) -> Result<(), String> {
        let targets = self.conflict_paths();
        self.resolve_conflicts_with_strategy(&targets, false)
    }

    fn resolve_conflicts_with_strategy(
        &mut self,
        paths: &[String],
        ours: bool,
    ) -> Result<(), String> {
        if paths.is_empty() {
            self.status_msg = "No conflict targets".to_string();
            return Ok(());
        }
        for path in paths {
            if ours {
                git_status(&["checkout", "--ours", "--", path])?;
            } else {
                git_status(&["checkout", "--theirs", "--", path])?;
            }
            git_status(&["add", "--", path])?;
        }
        let side = if ours { "ours" } else { "theirs" };
        self.status_msg = format!("Resolved {} conflict(s) with {side}", paths.len());
        self.refresh()?;
        Ok(())
    }

    pub(super) fn mark_conflicts_resolved_selected_or_marked(&mut self) -> Result<(), String> {
        let targets = self.conflict_targets_for_apply();
        if targets.is_empty() {
            self.status_msg = "No conflict targets".to_string();
            return Ok(());
        }
        for path in &targets {
            git_status(&["add", "--", path])?;
        }
        self.status_msg = format!("Marked {} conflict(s) resolved", targets.len());
        self.refresh()?;
        Ok(())
    }

    pub(super) fn abort_merge(&mut self) -> Result<(), String> {
        git_status(&["merge", "--abort"])?;
        self.status_msg = "Merge aborted".to_string();
        self.refresh()?;
        Ok(())
    }

    pub(super) fn select_next_conflict_pick(&mut self) {
        self.conflict_pick_selected = (self.conflict_pick_selected + 1) % 5;
    }

    pub(super) fn select_prev_conflict_pick(&mut self) {
        if self.conflict_pick_selected == 0 {
            self.conflict_pick_selected = 4;
        } else {
            self.conflict_pick_selected -= 1;
        }
    }

    pub(super) fn run_conflict_resolution_selected(&mut self) -> Result<(), String> {
        let Some(target) = self.conflict_target.clone() else {
            self.overlay = None;
            self.status_msg = "No conflict target selected".to_string();
            return Ok(());
        };
        self.overlay = None;
        match self.conflict_pick_selected {
            0 => {
                git_status(&["checkout", "--ours", "--", &target])?;
                git_status(&["add", "--", &target])?;
                self.status_msg = format!("Resolved (ours): {target}");
            }
            1 => {
                git_status(&["checkout", "--theirs", "--", &target])?;
                git_status(&["add", "--", &target])?;
                self.status_msg = format!("Resolved (theirs): {target}");
            }
            2 => {
                git_status(&["add", "--", &target])?;
                self.status_msg = format!("Marked resolved: {target}");
            }
            3 => {
                let _ = git_status(&["merge", "--abort"]);
                self.status_msg = "Requested merge --abort".to_string();
            }
            4 => {
                let _ = git_status(&["mergetool", "--", &target]);
                self.status_msg = format!("Opened mergetool for {target}");
            }
            _ => {}
        }
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
            "fetch" => self.fetch_all()?,
            "pull" => self.pull_current_branch()?,
            "stash" => {
                let message = cmdline.trim_start_matches("stash").trim();
                if message.is_empty() {
                    self.input_mode = InputMode::StashMessage;
                    self.input.clear();
                } else {
                    self.create_stash(message)?;
                }
            }
            "stashes" | "stash-tab" => {
                self.tab = Tab::Stashes;
                self.status_msg = "Switched to Stashes tab".to_string();
            }
            "stash-apply" => self.apply_selected_stash(false)?,
            "stash-pop" => self.apply_selected_stash(true)?,
            "reset" => self.open_reset_picker()?,
            "squash" => self.open_squash_picker()?,
            "ignore" => {
                let entry = cmdline.trim_start_matches("ignore").trim();
                if entry.is_empty() {
                    self.input_mode = InputMode::Gitignore;
                    self.input.clear();
                } else {
                    self.add_to_gitignore(entry)?;
                }
            }
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
                    self.open_branch_picker()?;
                } else {
                    git_status(&["switch", "-c", parts[1]])?;
                    self.status_msg = format!("Created and switched to {}", parts[1]);
                    self.refresh()?;
                }
            }
            "branches" | "branch-picker" | "branchpick" => self.open_branch_picker()?,
            "switch" => {
                if parts.len() < 2 {
                    self.open_branch_picker()?;
                } else {
                    git_status(&["switch", parts[1]])?;
                    self.status_msg = format!("Switched to {}", parts[1]);
                    self.refresh()?;
                }
            }
            "resolve-conflict" | "resolve" => self.open_conflict_resolver(),
            "conflicts" | "conflict-tab" => self.open_conflicts_tab(),
            "resolve-all-ours" => self.resolve_conflicts_all_ours()?,
            "resolve-all-theirs" => self.resolve_conflicts_all_theirs()?,
            "resolve-marked-ours" => self.resolve_conflicts_ours_selected_or_marked()?,
            "resolve-marked-theirs" => self.resolve_conflicts_theirs_selected_or_marked()?,
            "mark-resolved" => self.mark_conflicts_resolved_selected_or_marked()?,
            "abort-merge" => self.abort_merge()?,
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
            "repo-diff" | "repodiff" => {
                self.diff_mode = DiffMode::Repo;
                self.refresh_diff();
                self.status_msg = "Diff mode: full repo".to_string();
            }
            "file-diff" | "filediff" => {
                self.diff_mode = DiffMode::SelectedFile;
                self.refresh_diff();
                self.status_msg = "Diff mode: selected file".to_string();
            }
            "toggle-diff" | "togglediff" => self.toggle_diff_mode(),
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
            "conflicts-tab" => {
                self.open_conflicts_tab();
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
        self.refresh_detail_diff();
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
                label: "Fetch (all + prune)".to_string(),
                action: PaletteAction::Command("fetch"),
            },
            PaletteEntry {
                label: "Pull Current Branch (--rebase)".to_string(),
                action: PaletteAction::Command("pull"),
            },
            PaletteEntry {
                label: "Stash Working Changes".to_string(),
                action: PaletteAction::Command("stash"),
            },
            PaletteEntry {
                label: "Open Stashes Tab".to_string(),
                action: PaletteAction::Command("stashes"),
            },
            PaletteEntry {
                label: "Squash Recent Commits".to_string(),
                action: PaletteAction::Command("squash"),
            },
            PaletteEntry {
                label: "Reset Branch to Commit".to_string(),
                action: PaletteAction::Command("reset"),
            },
            PaletteEntry {
                label: "Add Selected Path to .gitignore".to_string(),
                action: PaletteAction::Command("ignore"),
            },
            PaletteEntry {
                label: "Push To Remote Branch".to_string(),
                action: PaletteAction::Command("push-remote"),
            },
            PaletteEntry {
                label: "Switch Branch (Picker)".to_string(),
                action: PaletteAction::Command("branches"),
            },
            PaletteEntry {
                label: "Resolve Merge Conflict".to_string(),
                action: PaletteAction::Command("resolve-conflict"),
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
                label: "Open Conflicts Tab".to_string(),
                action: PaletteAction::Command("conflicts"),
            },
            PaletteEntry {
                label: "Diff Mode: Selected File".to_string(),
                action: PaletteAction::Command("file-diff"),
            },
            PaletteEntry {
                label: "Diff Mode: Repo".to_string(),
                action: PaletteAction::Command("repo-diff"),
            },
            PaletteEntry {
                label: "Resolve All Conflicts (Ours)".to_string(),
                action: PaletteAction::Command("resolve-all-ours"),
            },
            PaletteEntry {
                label: "Resolve All Conflicts (Theirs)".to_string(),
                action: PaletteAction::Command("resolve-all-theirs"),
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
        let mut conflicts = Vec::new();
        let mut tracked = Vec::new();
        let mut untracked = Vec::new();
        let mut other = Vec::new();

        for (idx, file) in self.files.iter().enumerate() {
            if file.is_conflict() {
                conflicts.push(idx);
            } else if file.is_untracked() {
                untracked.push(idx);
            } else if file.is_tracked_change() {
                tracked.push(idx);
            } else {
                other.push(idx);
            }
        }

        let mut rows = Vec::new();
        if !conflicts.is_empty() {
            rows.push(StatusRow::Header(" MERGE CONFLICTS "));
            rows.extend(conflicts.into_iter().map(StatusRow::File));
            rows.push(StatusRow::Spacer);
        }
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

    pub(super) fn scroll_active_by(&mut self, delta: isize) -> bool {
        if delta == 0 {
            return false;
        }

        if self.overlay == Some(Overlay::Palette) {
            let before = self.palette_selected;
            if delta > 0 {
                for _ in 0..delta as usize {
                    self.select_next_palette();
                }
            } else {
                for _ in 0..(-delta) as usize {
                    self.select_prev_palette();
                }
            }
            return self.palette_selected != before;
        }

        if self.overlay == Some(Overlay::BranchPicker) {
            let before = self.branch_pick_selected;
            if delta > 0 {
                for _ in 0..delta as usize {
                    self.select_next_branch_pick();
                }
            } else {
                for _ in 0..(-delta) as usize {
                    self.select_prev_branch_pick();
                }
            }
            return self.branch_pick_selected != before;
        }

        if self.overlay == Some(Overlay::ConflictResolver) {
            let before = self.conflict_pick_selected;
            if delta > 0 {
                for _ in 0..delta as usize {
                    self.select_next_conflict_pick();
                }
            } else {
                for _ in 0..(-delta) as usize {
                    self.select_prev_conflict_pick();
                }
            }
            return self.conflict_pick_selected != before;
        }

        match self.tab {
            Tab::Workspace => {
                if self.pane != Pane::Diff {
                    return false;
                }
                let before = self.workspace_diff.scroll;
                if delta > 0 {
                    let max = self.workspace_diff_max_scroll();
                    self.workspace_diff.scroll = self
                        .workspace_diff
                        .scroll
                        .saturating_add(delta as usize)
                        .min(max);
                } else {
                    self.workspace_diff.scroll = self
                        .workspace_diff
                        .scroll
                        .saturating_sub((-delta) as usize);
                }
                self.workspace_diff.scroll != before
            }
            Tab::Graph => false,
            Tab::CommitDiff => {
                let before = self.detail_diff.scroll;
                if delta > 0 {
                    let max = self.detail_diff_max_scroll();
                    self.detail_diff.scroll = self
                        .detail_diff
                        .scroll
                        .saturating_add(delta as usize)
                        .min(max);
                } else {
                    self.detail_diff.scroll = self.detail_diff.scroll.saturating_sub((-delta) as usize);
                }
                self.detail_diff.scroll != before
            }
            Tab::Conflicts => {
                let before = self.conflict_selected;
                if delta > 0 {
                    let max = self.conflict_paths().len().saturating_sub(1);
                    self.conflict_selected = self
                        .conflict_selected
                        .saturating_add(delta as usize)
                        .min(max);
                } else {
                    self.conflict_selected =
                        self.conflict_selected.saturating_sub((-delta) as usize);
                }
                self.conflict_selected != before
            }
            Tab::Stashes => {
                let before = self.stash_selected;
                self.stash_selected = if delta > 0 {
                    next_index(self.stash_selected, self.stashes.len())
                } else {
                    prev_index(self.stash_selected, self.stashes.len())
                };
                self.stash_selected != before
            }
        }
    }

    fn workspace_diff_max_scroll(&self) -> usize {
        let rows = self.workspace_diff.view_rows.max(1);
        self.workspace_diff.rendered.len().saturating_sub(rows)
    }

    fn detail_diff_max_scroll(&self) -> usize {
        let rows = self.detail_diff.view_rows.max(1);
        self.detail_diff.rendered.len().saturating_sub(rows)
    }
}

fn next_index(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

fn prev_index(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else if current == 0 { len - 1 } else { current - 1 }
}

fn short_hash(hash: &str) -> &str {
    &hash[..hash.len().min(12)]
}

fn normalize_gitignore_entry(raw: &str) -> Result<String, String> {
    let entry = raw.trim().replace('\\', "/");
    if entry.is_empty() || entry.starts_with('#') || entry.contains('\n') || entry.contains('\r') {
        return Err("gitignore entry must be a non-empty single path or pattern".to_string());
    }
    if entry.starts_with('/') || entry.split('/').any(|part| part == "..") {
        return Err("gitignore entry must be repository-relative".to_string());
    }
    Ok(entry)
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
    "Cmds: help|cmdhelp|refresh|stage|unstage|stage-all|unstage-all|commit <msg>|push|fetch|pull|push-remote <remote>/<branch>|branch [name]|branches|switch [name]|resolve-conflict|conflicts|resolve-all-ours|resolve-all-theirs|resolve-marked-ours|resolve-marked-theirs|mark-resolved|abort-merge|workspace|graph-tab|commitdiff|conflicts-tab|diff|file-diff|repo-diff|toggle-diff|theme <name>|themes|palette|quit".to_string()
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
    process::ensure_command_exists("git", "dusk git tui")?;
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
    process::ensure_command_exists("git", "dusk git tui")?;
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
