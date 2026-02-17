use std::cmp;
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::*;

impl App {
    pub(super) fn render(&self, cursor_on: bool) -> Result<(), String> {
        let mut out = io::stdout();
        let (w, h) = crossterm::terminal::size().map_err(|e| e.to_string())?;
        let w = w as usize;
        let h = h as usize;

        queue!(
            out,
            BeginSynchronizedUpdate,
            MoveTo(0, 0),
            Clear(ClearType::All)
        )
        .map_err(|e| e.to_string())?;

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
        let hint_raw = "j/k move  1/2/3 tabs  s/u stage  A/U all  c commit  p push  R push-remote  t theme  Ctrl+P palette  ? help  q quit";
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
                    let diff = self.render_diff(w, diff_h);

                    let mut row = 3usize;
                    for line in files {
                        draw_line(&mut out, row as u16, &line)?;
                        row += 1;
                    }
                    for line in logs {
                        draw_line(&mut out, row as u16, &line)?;
                        row += 1;
                    }
                    for line in diff {
                        draw_line(&mut out, row as u16, &line)?;
                        row += 1;
                    }
                } else {
                    let left_w = cmp::min(cmp::max(36, w * 2 / 5), w.saturating_sub(24));
                    let right_w = w.saturating_sub(left_w + 1);
                    let log_h = body_h / 2;

                    let files = self.render_files(left_w, body_h);
                    let logs = self.render_log(right_w, log_h);
                    let diff = self.render_diff(right_w, body_h.saturating_sub(log_h));

                    for row in 0..body_h {
                        let left = files
                            .get(row)
                            .cloned()
                            .unwrap_or_else(|| " ".repeat(left_w));
                        let right = if row < log_h {
                            logs.get(row)
                                .cloned()
                                .unwrap_or_else(|| " ".repeat(right_w))
                        } else {
                            diff.get(row - log_h)
                                .cloned()
                                .unwrap_or_else(|| " ".repeat(right_w))
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
                let diff = self.render_commit_diff_tab(w, body_h);
                for (row, line) in diff.into_iter().enumerate() {
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
            "{}  {}  {}    {}",
            tab("1 Workspace", self.tab == Tab::Workspace),
            tab("2 Graph", self.tab == Tab::Graph),
            tab("3 CommitDiff", self.tab == Tab::CommitDiff),
            self.style.paint(
                self.theme.accent,
                "(:cmd, :cmdhelp, ? help, Ctrl+P palette)"
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

    fn render_diff(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(if self.pane == Pane::Diff {
            self.color_cell(" DIFF ", width, self.theme.ok)
        } else {
            self.color_cell(" DIFF ", width, self.theme.accent)
        });

        for line in self.diff_lines.iter().take(height.saturating_sub(1)) {
            lines.push(color_diff_line(self, line, width));
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

    fn render_commit_diff_tab(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        lines.push(self.color_cell(" COMMIT DIFF (selected commit) ", width, self.theme.ok));
        if self.commit_diff_lines.is_empty() {
            lines.push(self.color_cell("No commit diff loaded", width, self.theme.warn));
            while lines.len() < height {
                lines.push(" ".repeat(width));
            }
            return lines;
        }

        let rows = height.saturating_sub(1);
        for line in self
            .commit_diff_lines
            .iter()
            .skip(self.commit_diff_scroll)
            .take(rows)
        {
            lines.push(color_diff_line(self, line, width));
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
            InputMode::SwitchBranch => format!("switch branch: {}{cursor}", self.input),
            InputMode::PushRemote => {
                format!("push remote branch: {}{cursor}  (origin main)", self.input)
            }
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

    fn render_help_overlay(&self, out: &mut io::Stdout, w: usize, h: usize) -> Result<(), String> {
        let lines: Vec<(String, &'static str)> = vec![
            ("Navigation".to_string(), self.theme.ok),
            (
                "j/k or Up/Down move, g/G first/last".to_string(),
                self.theme.info,
            ),
            (
                "1 Workspace, 2 Graph, 3 CommitDiff".to_string(),
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
                "c commit, p push current, R push remote branch".to_string(),
                self.theme.info,
            ),
            (
                "b create branch, B switch branch".to_string(),
                self.theme.info,
            ),
            ("".to_string(), self.theme.info),
            ("Command Mode".to_string(), self.theme.ok),
            (
                "Use :cmdhelp for all commands and syntax".to_string(),
                self.theme.info,
            ),
            (
                ":workspace :graph-tab :commitdiff :theme <name> :themes".to_string(),
                self.theme.info,
            ),
            (
                ":push-remote <remote>/<branch> or <remote> <branch>".to_string(),
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
    queue!(
        out,
        MoveTo(0, y),
        Clear(ClearType::CurrentLine),
        Print(line)
    )
    .map_err(|e| e.to_string())
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

fn color_diff_line(app: &App, line: &str, width: usize) -> String {
    if line.starts_with("@@") {
        app.style.paint(app.theme.number, pad_display(line, width))
    } else if line.starts_with('+') && !line.starts_with("+++") {
        app.style.paint(app.theme.ok, pad_display(line, width))
    } else if line.starts_with('-') && !line.starts_with("---") {
        app.style.paint(app.theme.warn, pad_display(line, width))
    } else if line.starts_with("commit ")
        || line.starts_with("Author:")
        || line.starts_with("Date:")
        || line.starts_with("diff --git")
        || line.starts_with("index ")
    {
        app.style.paint(app.theme.accent, pad_display(line, width))
    } else {
        app.style.paint(app.theme.info, pad_display(line, width))
    }
}
