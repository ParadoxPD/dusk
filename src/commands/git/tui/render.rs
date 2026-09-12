use std::cmp;
use std::collections::HashSet;
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::*;

impl App {
    fn ensure_view_rendered(&mut self, detail: bool, width: usize) {
        let view = if detail {
            &mut self.detail_diff
        } else {
            &mut self.workspace_diff
        };
        if view.render_width != width || view.rendered.is_empty() {
            view.rendered = super::super::diffview::render_side_by_side(
                &view.lines.join("\n"),
                &self.style,
                self.theme,
                width,
            );
            view.render_width = width;
        }
    }

    fn diff_header(&self, width: usize) -> String {
        let mode = match self.diff_mode {
            DiffMode::SelectedFile => "selected-file",
            DiffMode::Repo => "repo",
        };
        if self.pane == Pane::Diff {
            self.color_cell(&format!(" DIFF [{mode}] "), width, self.theme.ok)
        } else {
            self.color_cell(&format!(" DIFF [{mode}] "), width, self.theme.accent)
        }
    }

    fn diff_window(&mut self, detail: bool, width: usize, height: usize) -> (usize, usize) {
        self.ensure_view_rendered(detail, width);
        let rows = height.saturating_sub(1);
        let view = if detail {
            &mut self.detail_diff
        } else {
            &mut self.workspace_diff
        };
        view.view_rows = rows;
        let max_scroll = view.rendered.len().saturating_sub(rows);
        if view.scroll > max_scroll {
            view.scroll = max_scroll;
        }
        (view.scroll, rows)
    }

    fn commit_diff_header(&self, width: usize) -> String {
        let sha = self
            .selected_commit
            .as_deref()
            .map(|s| s.chars().take(12).collect::<String>())
            .unwrap_or_else(|| "none".to_string());
        self.color_cell(&format!(" COMMIT DIFF ({sha}) "), width, self.theme.ok)
    }

    pub(super) fn render(&mut self, cursor_on: bool) -> Result<(), String> {
        let mut out = io::stdout();
        let (w, h) = crossterm::terminal::size().map_err(|e| e.to_string())?;
        let w = w as usize;
        let h = h as usize;

        queue!(out, BeginSynchronizedUpdate, MoveTo(0, 0)).map_err(|e| e.to_string())?;

        let title_raw = format!(
            "{} dusk git tui  {} {}  ⇄ {}  theme:{}",
            self.style.maybe_icon(icons::ICON_GIT),
            icons::ICON_BRANCH,
            self.branch,
            self.upstream.as_deref().unwrap_or("no-upstream"),
            self.theme.name
        );
        let title = self
            .style
            .paint(self.theme.title, pad_display(&title_raw, w));
        let hint_raw = "j/k move  1/2/3/4/5 tabs  r refresh  z stash  f fetch  L pull  B branches  Ctrl+P palette  ? help  q quit";
        let hint = self
            .style
            .paint(self.theme.subtle, pad_display(hint_raw, w));
        let tabs = self.render_tab_bar(w);

        draw_line(&mut out, 0, &title)?;
        draw_line(&mut out, 1, &hint)?;
        draw_line(&mut out, 2, &tabs)?;

        let body_h = h.saturating_sub(5);

        match self.tab {
            Tab::Workspace => {
                let compact = w < 100;
                if compact {
                    let status_h = cmp::max(6, body_h / 2);
                    let remain = body_h.saturating_sub(status_h);
                    let log_h = cmp::max(3, remain / 2);
                    let diff_h = body_h.saturating_sub(status_h + log_h);

                    let files = self.render_files(w, status_h);
                    let logs = self.render_log(w, log_h);
                    let diff_header = self.diff_header(w);
                    let (diff_start, diff_rows) = self.diff_window(false, w, diff_h);
                    let diff_blank = " ".repeat(w);

                    let mut row = 3usize;
                    for line in files {
                        draw_line(&mut out, row as u16, &line)?;
                        row += 1;
                    }
                    for line in logs {
                        draw_line(&mut out, row as u16, &line)?;
                        row += 1;
                    }

                    draw_line(&mut out, row as u16, &diff_header)?;
                    row += 1;
                    for i in 0..diff_rows {
                        let line = self
                            .workspace_diff
                            .rendered
                            .get(diff_start + i)
                            .map(String::as_str)
                            .unwrap_or(diff_blank.as_str());
                        draw_line(&mut out, row as u16, line)?;
                        row += 1;
                    }
                } else {
                    let left_w = cmp::min(cmp::max(36, w * 2 / 5), w.saturating_sub(24));
                    let right_w = w.saturating_sub(left_w + 1);
                    let log_h = body_h / 2;
                    let diff_h = body_h.saturating_sub(log_h);

                    let files = self.render_files(left_w, body_h);
                    let logs = self.render_log(right_w, log_h);
                    let diff_header = self.diff_header(right_w);
                    let (diff_start, diff_rows) = self.diff_window(false, right_w, diff_h);
                    let left_blank = " ".repeat(left_w);
                    let right_blank = " ".repeat(right_w);

                    for row in 0..body_h {
                        let left = files
                            .get(row)
                            .map(String::as_str)
                            .unwrap_or(left_blank.as_str());
                        let right = if row < log_h {
                            logs.get(row)
                                .map(String::as_str)
                                .unwrap_or(right_blank.as_str())
                        } else {
                            let drow = row - log_h;
                            if drow == 0 {
                                diff_header.as_str()
                            } else {
                                let idx = drow - 1;
                                if idx < diff_rows {
                                    self.workspace_diff
                                        .rendered
                                        .get(diff_start + idx)
                                        .map(String::as_str)
                                        .unwrap_or(right_blank.as_str())
                                } else {
                                    right_blank.as_str()
                                }
                            }
                        };
                        let sep = self.style.paint(self.theme.accent, "│");
                        draw_line(&mut out, (row + 3) as u16, &format!("{left}{sep}{right}"))?;
                    }
                }
            }
            Tab::Graph => {
                let graph = self.render_graph_tab(w, body_h);
                for (row, line) in graph.into_iter().enumerate() {
                    draw_line(&mut out, (row + 3) as u16, &line)?;
                }
            }
            Tab::CommitDiff => {
                self.render_detail_diff_tab(&mut out, w, body_h)?;
            }
            Tab::Conflicts => {
                let conflicts = self.render_conflicts_tab(w, body_h);
                for (row, line) in conflicts.into_iter().enumerate() {
                    draw_line(&mut out, (row + 3) as u16, &line)?;
                }
            }
            Tab::Stashes => {
                let stashes = self.render_stashes_tab(w, body_h);
                for (row, line) in stashes.into_iter().enumerate() {
                    draw_line(&mut out, (row + 3) as u16, &line)?;
                }
            }
        }

        let footer = self.render_status_line(w, cursor_on);
        draw_line(&mut out, h.saturating_sub(1) as u16, &footer)?;

        if let Some(overlay) = self.overlay {
            match overlay {
                Overlay::Help => self.render_help_overlay(&mut out, w, h)?,
                Overlay::Palette => self.render_palette_overlay(&mut out, w, h, cursor_on)?,
                Overlay::Push => self.render_push_overlay(&mut out, w, h)?,
                Overlay::BranchPicker => self.render_branch_picker_overlay(&mut out, w, h)?,
                Overlay::ConflictResolver => self.render_conflict_overlay(&mut out, w, h)?,
                Overlay::ResetPicker => self.render_reset_picker_overlay(&mut out, w, h)?,
                Overlay::SquashPicker => self.render_squash_picker_overlay(&mut out, w, h)?,
                Overlay::Confirm => self.render_confirm_overlay(&mut out, w, h)?,
            }
        }

        queue!(out, EndSynchronizedUpdate).map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())
    }

    fn color_cell(&self, text: &str, width: usize, color: &str) -> String {
        self.style.paint(color, pad_display(text, width))
    }

    fn render_tab_bar(&self, width: usize) -> String {
        let tab = |name: &str, active: bool| {
            if active {
                self.style.paint(self.theme.ok, format!("[{name}]"))
            } else {
                self.style.paint(self.theme.subtle, format!(" {name} "))
            }
        };
        let raw = format!(
            "{}  {}  {}  {}  {}    {}",
            tab("1 Workspace", self.tab == Tab::Workspace),
            tab("2 Graph", self.tab == Tab::Graph),
            tab("3 CommitDiff", self.tab == Tab::CommitDiff),
            tab("4 Conflicts", self.tab == Tab::Conflicts),
            tab("5 Stashes", self.tab == Tab::Stashes),
            self.style.paint(
                self.theme.accent,
                "(:cmd, :cmdhelp, :fetch, :pull, :branches, ? help, Ctrl+P palette)"
            )
        );
        pad_display(&raw, width)
    }

    fn render_files(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(if self.pane == Pane::Files {
            self.color_cell(" STATUS ", width, self.theme.ok)
        } else {
            self.color_cell(" STATUS ", width, self.theme.accent)
        });

        let rows = height.saturating_sub(1);
        if rows == 0 {
            return lines;
        }

        if self.files.is_empty() {
            lines.push(self.color_cell("  clean working tree", width, self.theme.info));
            while lines.len() < height {
                lines.push(" ".repeat(width));
            }
            return lines;
        }

        let grouped = self.status_rows();
        let selected_row = grouped
            .iter()
            .position(|r| matches!(r, StatusRow::File(idx) if *idx == self.selected))
            .unwrap_or(0);

        let start = selected_row.saturating_sub(rows.saturating_sub(1));
        for row in grouped.iter().skip(start).take(rows) {
            match row {
                StatusRow::Header(label) => {
                    lines.push(self.color_cell(label, width, self.theme.number))
                }
                StatusRow::Spacer => lines.push(self.color_cell("", width, self.theme.info)),
                StatusRow::File(idx) => {
                    let file = &self.files[*idx];
                    let icon = if file.is_untracked() {
                        icons::ICON_UNTRACKED
                    } else if file.x != ' ' {
                        icons::ICON_STAGED
                    } else {
                        icons::ICON_MODIFIED
                    };
                    let raw = format!(
                        " {} {} {}",
                        file.tag(),
                        self.style.maybe_icon(icon),
                        file.display_path
                    );
                    let painted = if *idx == self.selected {
                        self.color_cell(&raw, width, "\x1b[1;97;44m")
                    } else if file.is_untracked() {
                        self.color_cell(&raw, width, self.theme.accent)
                    } else if file.is_deleted() {
                        self.color_cell(&raw, width, self.theme.warn)
                    } else if file.is_modified() {
                        self.color_cell(&raw, width, self.theme.ok)
                    } else {
                        self.color_cell(&raw, width, self.theme.info)
                    };
                    lines.push(painted);
                }
            }
        }

        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_log(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(if self.pane == Pane::Log {
            self.color_cell(" LOG GRAPH ", width, self.theme.ok)
        } else {
            self.color_cell(" LOG GRAPH ", width, self.theme.accent)
        });

        for line in self.log_lines.iter().take(height.saturating_sub(1)) {
            lines.push(color_log_line(self, line, width));
        }
        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_graph_tab(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(self.color_cell(" FULL GIT GRAPH ", width, self.theme.ok));
        if self.log_lines.is_empty() {
            lines.push(self.color_cell("No commits available", width, self.theme.warn));
            while lines.len() < height {
                lines.push(" ".repeat(width));
            }
            return lines;
        }

        let rows = height.saturating_sub(1);
        let start = self.log_selected.saturating_sub(rows.saturating_sub(1));
        for (offset, line) in self.log_lines.iter().skip(start).take(rows).enumerate() {
            let idx = start + offset;
            if idx == self.log_selected {
                lines.push(self.color_cell(line, width, "\x1b[1;97;44m"));
            } else {
                lines.push(color_log_line(self, line, width));
            }
        }
        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_detail_diff_tab(
        &mut self,
        out: &mut io::Stdout,
        width: usize,
        height: usize,
    ) -> Result<(), String> {
        let label = match self.detail_diff_mode {
            DetailDiffMode::Commit => self
                .selected_commit
                .as_deref()
                .map(|sha| format!(" COMMIT DIFF ({})  d: mode ", &sha[..sha.len().min(12)]))
                .unwrap_or_else(|| " COMMIT DIFF (none)  d: mode ".to_string()),
            DetailDiffMode::Repo => " REPOSITORY DIFF  d: mode ".to_string(),
            DetailDiffMode::SelectedFile => " FILE DIFF  d: mode, h/l: files/diff ".to_string(),
        };

        if self.detail_diff_mode != DetailDiffMode::SelectedFile || width < 88 {
            let header = self.color_cell(&label, width, self.theme.ok);
            draw_line(out, 3, &header)?;
            let (start, rows) = self.diff_window(true, width, height);
            let blank = " ".repeat(width);
            for i in 0..rows {
                let line = self
                    .detail_diff
                    .rendered
                    .get(start + i)
                    .map(String::as_str)
                    .unwrap_or(blank.as_str());
                draw_line(out, (4 + i) as u16, line)?;
            }
            return Ok(());
        }

        let selector_w = cmp::min(32, width / 3).max(20);
        let diff_w = width.saturating_sub(selector_w + 1);
        let selector = self.render_compact_file_selector(selector_w, height);
        let header = self.color_cell(&label, diff_w, self.theme.ok);
        let (start, rows) = self.diff_window(true, diff_w, height);
        let left_blank = " ".repeat(selector_w);
        let right_blank = " ".repeat(diff_w);

        for row in 0..height {
            let left = selector
                .get(row)
                .map(String::as_str)
                .unwrap_or(left_blank.as_str());
            let right = if row == 0 {
                header.as_str()
            } else if row - 1 < rows {
                self.detail_diff
                    .rendered
                    .get(start + row - 1)
                    .map(String::as_str)
                    .unwrap_or(right_blank.as_str())
            } else {
                right_blank.as_str()
            };
            let sep = self.style.paint(self.theme.accent, "│");
            draw_line(out, (row + 3) as u16, &format!("{left}{sep}{right}"))?;
        }
        Ok(())
    }

    fn render_compact_file_selector(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let active = self.detail_pane == DetailPane::Files;
        lines.push(self.color_cell(
            " CHANGED FILES ",
            width,
            if active { self.theme.ok } else { self.theme.accent },
        ));
        let order = self.file_selection_order();
        if order.is_empty() {
            lines.push(self.color_cell(" clean ", width, self.theme.info));
        } else {
            let selected_pos = order.iter().position(|idx| *idx == self.selected).unwrap_or(0);
            let start = selected_pos.saturating_sub(height.saturating_sub(2));
            for idx in order.iter().skip(start).take(height.saturating_sub(1)) {
                let file = &self.files[*idx];
                let row = format!("{} {}", file.tag(), file.display_path);
                let color = if *idx == self.selected {
                    "\x1b[1;97;44m"
                } else {
                    self.theme.info
                };
                lines.push(self.color_cell(&row, width, color));
            }
        }
        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_status_line(&self, width: usize, cursor_on: bool) -> String {
        let cursor = if cursor_on { "▍" } else { " " };
        let mode = match self.input_mode {
            InputMode::None => String::new(),
            InputMode::Commit => format!("commit msg: {}{cursor}", self.input),
            InputMode::NewBranch => format!("new branch: {}{cursor}", self.input),
            InputMode::PushRemote => {
                format!("push remote branch: {}{cursor}  (origin main)", self.input)
            }
            InputMode::StashMessage => format!("stash message (optional): {}{cursor}", self.input),
            InputMode::Gitignore => format!(".gitignore entry: {}{cursor}", self.input),
            InputMode::SquashMessage => format!("squash commit message: {}{cursor}", self.input),
            InputMode::Command => format!(":{}{}", self.input, cursor),
        };

        if mode.is_empty() {
            self.style
                .paint(self.theme.info, pad_display(&self.status_msg, width))
        } else {
            self.style
                .paint(self.theme.accent, pad_display(&mode, width))
        }
    }

    fn render_conflicts_tab(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(self.color_cell(
            " CONFLICTS (space mark, a mark-all, o/i ours/theirs, O/I all ours/theirs, m add, x abort) ",
            width,
            self.theme.ok,
        ));
        let conflicts = self.conflict_paths();
        if conflicts.is_empty() {
            lines.push(self.color_cell("No merge conflicts", width, self.theme.info));
            while lines.len() < height {
                lines.push(" ".repeat(width));
            }
            return lines;
        }
        let rows = height.saturating_sub(1);
        let start = self
            .conflict_selected
            .saturating_sub(rows.saturating_sub(1));
        for (offset, path) in conflicts.iter().skip(start).take(rows).enumerate() {
            let idx = start + offset;
            let mark = if self.conflict_marked.contains(path) {
                "*"
            } else {
                " "
            };
            let line = format!("{mark} {path}");
            if idx == self.conflict_selected {
                lines.push(self.color_cell(&line, width, "\x1b[1;97;44m"));
            } else if self.conflict_marked.contains(path) {
                lines.push(self.color_cell(&line, width, self.theme.accent));
            } else {
                lines.push(self.color_cell(&line, width, self.theme.warn));
            }
        }
        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_stashes_tab(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(self.color_cell(
            " STASHES (z create, a apply, p pop, j/k select) ",
            width,
            self.theme.ok,
        ));
        if self.stashes.is_empty() {
            lines.push(self.color_cell("No stashes", width, self.theme.info));
        } else {
            let rows = height.saturating_sub(1);
            let start = self.stash_selected.saturating_sub(rows.saturating_sub(1));
            for (offset, stash) in self.stashes.iter().skip(start).take(rows).enumerate() {
                let idx = start + offset;
                let text = format!("{:<10} {:<14} {}", stash.reference, stash.age, stash.message);
                let color = if idx == self.stash_selected { "\x1b[1;97;44m" } else { self.theme.info };
                lines.push(self.color_cell(&text, width, color));
            }
        }
        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_help_overlay(&self, out: &mut io::Stdout, w: usize, h: usize) -> Result<(), String> {
        let lines: Vec<(String, &'static str)> = vec![
            ("Navigation".to_string(), self.theme.ok),
            (
                "j/k or Up/Down move, g/G first/last".to_string(),
                self.theme.info,
            ),
            (
                "1 Workspace, 2 Graph, 3 DetailDiff, 4 Conflicts, 5 Stashes".to_string(),
                self.theme.info,
            ),
            (
                "h/l or Left/Right/Tab switch pane (workspace tab)".to_string(),
                self.theme.info,
            ),
            ("".to_string(), self.theme.info),
            ("Git Actions".to_string(), self.theme.ok),
            (
                "s/u stage or unstage selected file".to_string(),
                self.theme.info,
            ),
            ("A/U stage all or unstage all".to_string(), self.theme.info),
            (
                "c commit, p push current, R push remote branch, f fetch, L pull".to_string(),
                self.theme.info,
            ),
            (
                "b create branch, B branch picker, m conflict resolver, D toggle repo/file diff"
                    .to_string(),
                self.theme.info,
            ),
            (
                "In Conflicts tab: space mark, a mark-all, o/i ours/theirs, O/I all, x abort"
                    .to_string(),
                self.theme.info,
            ),
            ("".to_string(), self.theme.info),
            ("Command Mode".to_string(), self.theme.ok),
            (
                "Use :cmdhelp for all commands and syntax".to_string(),
                self.theme.info,
            ),
            (
                ":workspace :graph-tab :commitdiff :file-diff :repo-diff".to_string(),
                self.theme.info,
            ),
            (
                ":fetch :pull :branches :resolve-conflict :push-remote <remote>/<branch>"
                    .to_string(),
                self.theme.info,
            ),
            ("".to_string(), self.theme.info),
            ("Tools".to_string(), self.theme.ok),
            (
                "t cycle theme, Ctrl+P/P palette, Esc closes overlays".to_string(),
                self.theme.info,
            ),
            ("Esc or ? closes help".to_string(), self.theme.accent),
        ];
        self.draw_center_overlay(out, w, h, " Help ", &lines)
    }

    fn render_palette_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
        cursor_on: bool,
    ) -> Result<(), String> {
        let entries = self.filtered_palette_entries();
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        let cursor = if cursor_on { "▍" } else { " " };
        lines.push((
            format!("Query: {}{}", self.palette_query, cursor),
            if self.palette_query.is_empty() {
                self.theme.subtle
            } else {
                self.theme.accent
            },
        ));
        lines.push((
            "j/k move, Enter run, Backspace edit, Esc close".to_string(),
            self.theme.subtle,
        ));
        lines.push(("".to_string(), self.theme.info));

        if entries.is_empty() {
            lines.push(("No commands match query".to_string(), self.theme.warn));
        } else {
            let max_items = 10usize;
            let window_start = if self.palette_selected >= max_items {
                self.palette_selected + 1 - max_items
            } else {
                0
            };
            for (i, item) in entries
                .iter()
                .skip(window_start)
                .take(max_items)
                .enumerate()
            {
                let actual = window_start + i;
                let prefix = if actual == self.palette_selected {
                    ">"
                } else {
                    " "
                };
                let color = if actual == self.palette_selected {
                    self.theme.ok
                } else if item.label.starts_with("Theme:") {
                    self.theme.accent
                } else {
                    self.theme.info
                };
                lines.push((format!("{prefix} {}", item.label), color));
            }
            if entries.len() > max_items && window_start + max_items < entries.len() {
                lines.push((
                    format!(
                        "… {} more entries",
                        entries.len() - (window_start + max_items)
                    ),
                    self.theme.subtle,
                ));
            }
        }

        self.draw_center_overlay(out, w, h, " Command Palette ", &lines)
    }

    fn render_push_overlay(&self, out: &mut io::Stdout, w: usize, h: usize) -> Result<(), String> {
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        let status_color = match self.push_overlay_ok {
            Some(true) => self.theme.ok,
            Some(false) => self.theme.warn,
            None => self.theme.accent,
        };
        let status_text = match self.push_overlay_ok {
            Some(true) => "Status: success",
            Some(false) => "Status: failed",
            None => "Status: running",
        };
        lines.push((status_text.to_string(), status_color));
        lines.push(("".to_string(), self.theme.info));

        for line in &self.push_overlay_lines {
            let color = if line.starts_with("$ git ") {
                self.theme.number
            } else if line.contains("failed") || line.contains("error") {
                self.theme.warn
            } else if line.contains("success") || line.contains("completed") {
                self.theme.ok
            } else {
                self.theme.info
            };
            lines.push((line.clone(), color));
        }

        self.draw_center_overlay(
            out,
            w,
            h,
            &format!(" {} ", self.action_overlay_title),
            &lines,
        )
    }

    fn render_branch_picker_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        lines.push((
            "j/k move, Enter switch, Esc close".to_string(),
            self.theme.subtle,
        ));
        lines.push(("".to_string(), self.theme.info));
        if self.branch_choices.is_empty() {
            lines.push(("No branches found".to_string(), self.theme.warn));
        } else {
            let max_items = 14usize;
            let window_start = if self.branch_pick_selected >= max_items {
                self.branch_pick_selected + 1 - max_items
            } else {
                0
            };
            for (i, name) in self
                .branch_choices
                .iter()
                .skip(window_start)
                .take(max_items)
                .enumerate()
            {
                let idx = window_start + i;
                let prefix = if idx == self.branch_pick_selected {
                    ">"
                } else {
                    " "
                };
                let marker = if name == &self.branch { "*" } else { " " };
                let color = if idx == self.branch_pick_selected {
                    self.theme.ok
                } else if marker == "*" {
                    self.theme.accent
                } else {
                    self.theme.info
                };
                lines.push((format!("{prefix} {marker} {name}"), color));
            }
        }
        self.draw_center_overlay(out, w, h, " Branches ", &lines)
    }

    fn render_conflict_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        let target = self.conflict_target.as_deref().unwrap_or("<none>");
        lines.push((format!("Target: {target}"), self.theme.number));
        lines.push((
            "j/k move, Enter apply, Esc close".to_string(),
            self.theme.subtle,
        ));
        lines.push(("".to_string(), self.theme.info));
        let opts = [
            "Use OURS and mark resolved",
            "Use THEIRS and mark resolved",
            "Mark resolved (git add)",
            "Abort merge (git merge --abort)",
            "Open mergetool for file",
        ];
        for (idx, label) in opts.iter().enumerate() {
            let prefix = if idx == self.conflict_pick_selected {
                ">"
            } else {
                " "
            };
            let color = if idx == self.conflict_pick_selected {
                self.theme.ok
            } else {
                self.theme.info
            };
            lines.push((format!("{prefix} {label}"), color));
        }
        self.draw_center_overlay(out, w, h, " Merge Conflict Resolver ", &lines)
    }

    fn render_reset_picker_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let mut lines = vec![
            (
                format!("Mode: {}  (s soft, h hard)", if self.reset_hard { "HARD" } else { "SOFT" }),
                if self.reset_hard { self.theme.warn } else { self.theme.ok },
            ),
            ("j/k select, Enter review, Esc close".to_string(), self.theme.subtle),
            ("".to_string(), self.theme.info),
        ];
        append_history_rows(&mut lines, &self.history_choices, self.reset_selected, None, self.theme);
        self.draw_center_overlay(out, w, h, " Reset Commit Selector ", &lines)
    }

    fn render_squash_picker_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let mut lines = vec![
            ("Space mark commits. Mark a contiguous range ending at HEAD.".to_string(), self.theme.subtle),
            ("Enter writes squash message. Esc closes.".to_string(), self.theme.subtle),
            ("".to_string(), self.theme.info),
        ];
        append_history_rows(
            &mut lines,
            &self.history_choices,
            self.reset_selected,
            Some(&self.squash_marked),
            self.theme,
        );
        self.draw_center_overlay(out, w, h, " Squash Commits ", &lines)
    }

    fn render_confirm_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let text = match &self.pending_action {
            Some(PendingAction::Reset { target, hard }) => format!(
                "{} reset to {}. {}",
                if *hard { "Hard" } else { "Soft" },
                &target[..target.len().min(12)],
                if *hard { "Worktree changes will be discarded." } else { "Index and worktree stay intact." }
            ),
            Some(PendingAction::Squash { oldest, message }) => format!(
                "Squash from {} into one commit: {}",
                &oldest[..oldest.len().min(12)],
                message
            ),
            None => "No pending action".to_string(),
        };
        self.draw_center_overlay(
            out,
            w,
            h,
            " Confirm History Rewrite ",
            &[
                (text, self.theme.warn),
                ("Enter/y confirm, Esc/n cancel".to_string(), self.theme.accent),
            ],
        )
    }

    fn draw_center_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
        title: &str,
        lines: &[(String, &'static str)],
    ) -> Result<(), String> {
        if w < 20 || h < 8 {
            return Ok(());
        }

        let inner_w = cmp::min(88, w.saturating_sub(8));
        let box_w = inner_w + 2;
        let max_inner_h = h.saturating_sub(6);
        let body_lines = cmp::min(lines.len(), max_inner_h.saturating_sub(3));
        let box_h = body_lines + 3;
        let x = (w.saturating_sub(box_w)) / 2;
        let y = (h.saturating_sub(box_h)) / 2;

        draw_at(
            out,
            x as u16,
            y as u16,
            &self
                .style
                .paint(self.theme.accent, format!("┌{}┐", "─".repeat(inner_w))),
        )?;

        draw_at(
            out,
            x as u16,
            (y + 1) as u16,
            &format!(
                "{}{}{}",
                self.style.paint(self.theme.accent, "│"),
                self.style
                    .paint(self.theme.title, pad_display(title, inner_w)),
                self.style.paint(self.theme.accent, "│")
            ),
        )?;

        for (i, (line, color)) in lines.iter().take(body_lines).enumerate() {
            draw_at(
                out,
                x as u16,
                (y + 2 + i) as u16,
                &format!(
                    "{}{}{}",
                    self.style.paint(self.theme.accent, "│"),
                    self.style.paint(*color, pad_display(line, inner_w)),
                    self.style.paint(self.theme.accent, "│")
                ),
            )?;
        }

        draw_at(
            out,
            x as u16,
            (y + box_h - 1) as u16,
            &self
                .style
                .paint(self.theme.accent, format!("└{}┘", "─".repeat(inner_w))),
        )?;
        Ok(())
    }
}

fn draw_line(out: &mut io::Stdout, y: u16, line: &str) -> Result<(), String> {
    queue!(out, MoveTo(0, y), Print(line)).map_err(|e| e.to_string())
}

fn draw_at(out: &mut io::Stdout, x: u16, y: u16, line: &str) -> Result<(), String> {
    queue!(out, MoveTo(x, y), Print(line)).map_err(|e| e.to_string())
}

fn pad_display(s: &str, width: usize) -> String {
    let clipped = truncate_display(s, width);
    let used = UnicodeWidthStr::width(clipped.as_str());
    if used >= width {
        clipped
    } else {
        format!("{clipped}{}", " ".repeat(width - used))
    }
}

fn truncate_display(s: &str, max: usize) -> String {
    if max <= 1 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + cw + 1 > max {
            out.push('…');
            return out;
        }
        out.push(ch);
        used += cw;
    }
    out
}

fn color_log_line(app: &App, line: &str, width: usize) -> String {
    let color = if line.contains('*') {
        app.theme.accent
    } else {
        app.theme.info
    };
    app.style.paint(color, pad_display(line, width))
}

fn append_history_rows(
    lines: &mut Vec<(String, &'static str)>,
    entries: &[HistoryEntry],
    selected: usize,
    marked: Option<&HashSet<String>>,
    theme: Theme,
) {
    let max_rows = 12usize;
    let start = selected.saturating_sub(max_rows.saturating_sub(1));
    for (offset, entry) in entries.iter().skip(start).take(max_rows).enumerate() {
        let idx = start + offset;
        let marker = marked
            .map(|set| if set.contains(&entry.hash) { "*" } else { " " })
            .unwrap_or(" ");
        let prefix = if idx == selected { ">" } else { " " };
        let hash = &entry.hash[..entry.hash.len().min(10)];
        let color = if idx == selected { theme.ok } else if marker == "*" { theme.accent } else { theme.info };
        lines.push((format!("{prefix}{marker} {hash} {}", entry.subject), color));
    }
}
