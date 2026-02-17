use std::cmp;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    self, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::core::icons;
use crate::core::style::Style;
use crate::core::theme::{self, THEMES, Theme};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Files,
    Log,
    Diff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InputMode {
    None,
    Commit,
    NewBranch,
    SwitchBranch,
    Command,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    Help,
    Palette,
}

#[derive(Clone)]
enum PaletteAction {
    Command(&'static str),
    Theme(&'static str),
}

#[derive(Clone)]
struct PaletteEntry {
    label: String,
    action: PaletteAction,
}

#[derive(Clone, Debug)]
struct FileStatus {
    x: char,
    y: char,
    display_path: String,
    git_path: String,
}

impl FileStatus {
    fn tag(&self) -> &'static str {
        match (self.x, self.y) {
            ('?', '?') => "??",
            ('A', _) => "A ",
            ('M', ' ') | ('M', _) => "M ",
            (' ', 'M') => " M",
            ('R', _) => "R ",
            ('D', _) => "D ",
            ('U', _) | (_, 'U') => "U ",
            _ => "  ",
        }
    }

    fn is_untracked(&self) -> bool {
        self.x == '?'
    }

    fn is_deleted(&self) -> bool {
        self.x == 'D' || self.y == 'D'
    }

    fn is_modified(&self) -> bool {
        self.x == 'M' || self.y == 'M'
    }

    fn is_tracked_change(&self) -> bool {
        !self.is_untracked() && (self.is_modified() || self.is_deleted())
    }
}

enum StatusRow {
    Header(&'static str),
    Spacer,
    File(usize),
}

struct App {
    theme: Theme,
    style: Style,
    pane: Pane,
    input_mode: InputMode,
    input: String,
    overlay: Option<Overlay>,
    palette_query: String,
    palette_selected: usize,
    status_msg: String,
    branch: String,
    files: Vec<FileStatus>,
    selected: usize,
    log_lines: Vec<String>,
    diff_lines: Vec<String>,
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|e| format!("failed to enable raw mode: {e}"))?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)
            .map_err(|e| format!("failed to enter alternate screen: {e}"))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run(theme_name: Option<&str>) -> Result<(), String> {
    let mut app = App::new(theme::active(theme_name), Style::for_stdout());
    app.refresh()?;

    let _guard = TerminalGuard::enter()?;
    let mut dirty = true;

    loop {
        if dirty {
            app.render()?;
            dirty = false;
        }

        if !event::poll(Duration::from_millis(250)).map_err(|e| e.to_string())? {
            continue;
        }

        match event::read().map_err(|e| e.to_string())? {
            Event::Key(key) => {
                if app.handle_key(key)? {
                    break;
                }
                dirty = true;
            }
            Event::Resize(_, _) => dirty = true,
            _ => {}
        }
    }

    Ok(())
}

impl App {
    fn new(theme: Theme, style: Style) -> Self {
        Self {
            theme,
            style,
            pane: Pane::Files,
            input_mode: InputMode::None,
            input: String::new(),
            overlay: None,
            palette_query: String::new(),
            palette_selected: 0,
            status_msg: "Press ? for help".to_string(),
            branch: String::new(),
            files: Vec::new(),
            selected: 0,
            log_lines: Vec::new(),
            diff_lines: Vec::new(),
        }
    }

    fn refresh(&mut self) -> Result<(), String> {
        self.branch = git_capture(&["branch", "--show-current"])?
            .trim()
            .to_string();
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

        self.refresh_diff();
        Ok(())
    }

    fn refresh_diff(&mut self) {
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

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }

        if self.input_mode != InputMode::None {
            return self.handle_input_key(key);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
            self.open_palette();
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            KeyCode::Char('P') => self.open_palette(),
            KeyCode::Char('r') => {
                self.refresh()?;
                self.status_msg = "Refreshed".to_string();
            }
            KeyCode::Char('t') => self.cycle_theme(),
            KeyCode::Char('h') | KeyCode::Left => self.pane = prev_pane(self.pane),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => self.pane = next_pane(self.pane),
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => self.selected = self.files.len().saturating_sub(1),
            KeyCode::Char('s') => self.stage_selected()?,
            KeyCode::Char('u') => self.unstage_selected()?,
            KeyCode::Char('A') => self.stage_all()?,
            KeyCode::Char('U') => self.unstage_all()?,
            KeyCode::Char('c') => {
                self.input_mode = InputMode::Commit;
                self.input.clear();
            }
            KeyCode::Char('p') => self.push_current_branch()?,
            KeyCode::Char('b') => {
                self.input_mode = InputMode::NewBranch;
                self.input.clear();
            }
            KeyCode::Char('B') => {
                self.input_mode = InputMode::SwitchBranch;
                self.input.clear();
            }
            KeyCode::Char(':') => {
                self.input_mode = InputMode::Command;
                self.input.clear();
            }
            _ => {}
        }

        if matches!(
            key.code,
            KeyCode::Char('j')
                | KeyCode::Down
                | KeyCode::Char('k')
                | KeyCode::Up
                | KeyCode::Char('g')
                | KeyCode::Char('G')
                | KeyCode::Char('s')
                | KeyCode::Char('u')
                | KeyCode::Char('A')
                | KeyCode::Char('U')
                | KeyCode::Char('r')
        ) {
            self.refresh_diff();
        }

        Ok(false)
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        match self.overlay {
            Some(Overlay::Help) => match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter | KeyCode::Char('q') => {
                    self.overlay = None;
                }
                _ => {}
            },
            Some(Overlay::Palette) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                }
                KeyCode::Enter => return self.execute_palette_selection(),
                KeyCode::Backspace => {
                    self.palette_query.pop();
                    self.palette_selected = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.palette_selected = self.palette_selected.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.filtered_palette_entries().len();
                    if len > 0 {
                        self.palette_selected = cmp::min(self.palette_selected + 1, len - 1);
                    }
                }
                KeyCode::Char(ch) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.palette_query.push(ch);
                        self.palette_selected = 0;
                    }
                }
                _ => {}
            },
            None => {}
        }
        Ok(false)
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.input.clear();
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                let input = self.input.trim().to_string();
                match self.input_mode {
                    InputMode::Commit => {
                        if input.is_empty() {
                            self.status_msg = "Commit message cannot be empty".to_string();
                        } else {
                            git_status(&["commit", "-m", &input])?;
                            self.status_msg = format!("Committed: {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::NewBranch => {
                        if input.is_empty() {
                            self.status_msg = "Branch name cannot be empty".to_string();
                        } else {
                            git_status(&["switch", "-c", &input])?;
                            self.status_msg = format!("Created and switched to {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::SwitchBranch => {
                        if input.is_empty() {
                            self.status_msg = "Branch name cannot be empty".to_string();
                        } else {
                            git_status(&["switch", &input])?;
                            self.status_msg = format!("Switched to {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::Command => {
                        if self.run_command(&input)? {
                            return Ok(true);
                        }
                    }
                    InputMode::None => {}
                }
                self.input_mode = InputMode::None;
                self.input.clear();
            }
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.push(ch);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn run_command(&mut self, cmdline: &str) -> Result<bool, String> {
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
                self.status_msg = "Focused log pane".to_string();
            }
            "diff" => {
                self.pane = Pane::Diff;
                self.status_msg = "Focused diff pane".to_string();
            }
            "status" => {
                self.pane = Pane::Files;
                self.status_msg = "Focused status pane".to_string();
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
        Ok(false)
    }

    fn open_palette(&mut self) {
        self.overlay = Some(Overlay::Palette);
        self.palette_query.clear();
        self.palette_selected = 0;
        self.status_msg = "Command palette opened".to_string();
    }

    fn cycle_theme(&mut self) {
        let idx = THEMES
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        let next = THEMES[(idx + 1) % THEMES.len()];
        self.theme = next;
        self.status_msg = format!("Theme: {}", self.theme.name);
    }

    fn apply_theme(&mut self, name: &str) -> bool {
        if let Some(found) = THEMES.iter().find(|t| t.name.eq_ignore_ascii_case(name)) {
            self.theme = *found;
            true
        } else {
            false
        }
    }

    fn palette_entries(&self) -> Vec<PaletteEntry> {
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
                label: "Focus Status Pane".to_string(),
                action: PaletteAction::Command("status"),
            },
            PaletteEntry {
                label: "Focus Log Pane".to_string(),
                action: PaletteAction::Command("log"),
            },
            PaletteEntry {
                label: "Focus Diff Pane".to_string(),
                action: PaletteAction::Command("diff"),
            },
            PaletteEntry {
                label: "Show Help".to_string(),
                action: PaletteAction::Command("help"),
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

    fn filtered_palette_entries(&self) -> Vec<PaletteEntry> {
        let q = self.palette_query.trim().to_lowercase();
        if q.is_empty() {
            return self.palette_entries();
        }
        self.palette_entries()
            .into_iter()
            .filter(|e| e.label.to_lowercase().contains(&q))
            .collect()
    }

    fn execute_palette_selection(&mut self) -> Result<bool, String> {
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

    fn stage_selected(&mut self) -> Result<(), String> {
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

    fn unstage_selected(&mut self) -> Result<(), String> {
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

    fn stage_all(&mut self) -> Result<(), String> {
        git_status(&["add", "-A"])?;
        self.status_msg = "Staged all changes".to_string();
        self.refresh()?;
        Ok(())
    }

    fn unstage_all(&mut self) -> Result<(), String> {
        git_status(&["restore", "--staged", "."])?;
        self.status_msg = "Unstaged all files".to_string();
        self.refresh()?;
        Ok(())
    }

    fn push_current_branch(&mut self) -> Result<(), String> {
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

    fn move_up(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.selected = cmp::min(self.selected + 1, self.files.len().saturating_sub(1));
    }

    fn render(&self) -> Result<(), String> {
        let mut out = io::stdout();
        let (w, h) = terminal::size().map_err(|e| e.to_string())?;
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
            "{} dusk git tui  {} {}  theme:{}",
            self.style.maybe_icon(icons::ICON_GIT),
            icons::ICON_BRANCH,
            self.branch,
            self.theme.name
        );
        let title = self
            .style
            .paint(self.theme.title, pad_display(&title_raw, w));
        let hint_raw = "j/k move  h/l panes  s/u stage  A/U all  c commit  p push  t cycle-theme  Ctrl+P palette  ? help  q quit";
        let hint = self
            .style
            .paint(self.theme.subtle, pad_display(hint_raw, w));

        draw_line(&mut out, 0, &title)?;
        draw_line(&mut out, 1, &hint)?;

        let body_h = h.saturating_sub(4);
        let compact = w < 100;

        if compact {
            let status_h = cmp::max(6, body_h / 2);
            let remain = body_h.saturating_sub(status_h);
            let log_h = cmp::max(3, remain / 2);
            let diff_h = body_h.saturating_sub(status_h + log_h);

            let files = self.render_files(w, status_h);
            let logs = self.render_log(w, log_h);
            let diff = self.render_diff(w, diff_h);

            let mut row = 2usize;
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
                draw_line(&mut out, (row + 2) as u16, &format!("{left}{sep}{right}"))?;
            }
        }

        let footer = self.render_status_line(w);
        draw_line(&mut out, h.saturating_sub(1) as u16, &footer)?;

        if let Some(overlay) = self.overlay {
            match overlay {
                Overlay::Help => self.render_help_overlay(&mut out, w, h)?,
                Overlay::Palette => self.render_palette_overlay(&mut out, w, h)?,
            }
        }

        queue!(out, EndSynchronizedUpdate).map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())
    }

    fn color_cell(&self, text: &str, width: usize, color: &str) -> String {
        self.style.paint(color, pad_display(text, width))
    }

    fn status_rows(&self) -> Vec<StatusRow> {
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
                    lines.push(self.color_cell(label, width, self.theme.number));
                }
                StatusRow::Spacer => {
                    lines.push(self.color_cell("", width, self.theme.info));
                }
                StatusRow::File(idx) => {
                    let file = &self.files[*idx];
                    let icon = if file.is_untracked() {
                        icons::ICON_UNTRACKED
                    } else if file.is_deleted() {
                        icons::ICON_MODIFIED
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
            let painted = if line.starts_with("@@") {
                self.color_cell(line, width, self.theme.number)
            } else if line.starts_with('+') && !line.starts_with("+++") {
                self.color_cell(line, width, self.theme.ok)
            } else if line.starts_with('-') && !line.starts_with("---") {
                self.color_cell(line, width, self.theme.warn)
            } else if line.starts_with("diff --git") || line.starts_with("index ") {
                self.color_cell(line, width, self.theme.accent)
            } else {
                self.color_cell(line, width, self.theme.info)
            };
            lines.push(painted);
        }

        while lines.len() < height {
            lines.push(" ".repeat(width));
        }
        lines
    }

    fn render_status_line(&self, width: usize) -> String {
        let mode = match self.input_mode {
            InputMode::None => String::new(),
            InputMode::Commit => format!("commit msg: {}", self.input),
            InputMode::NewBranch => format!("new branch: {}", self.input),
            InputMode::SwitchBranch => format!("switch branch: {}", self.input),
            InputMode::Command => format!(":{}", self.input),
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
                "j/k or Up/Down move selection, g/G first/last".to_string(),
                self.theme.info,
            ),
            (
                "h/l or Left/Right/Tab switch panes".to_string(),
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
                "c commit, p push, b create branch, B switch branch".to_string(),
                self.theme.info,
            ),
            ("".to_string(), self.theme.info),
            ("Tools".to_string(), self.theme.ok),
            (
                "t cycle theme, Ctrl+P command palette, : command mode".to_string(),
                self.theme.info,
            ),
            ("Esc or ? closes this help".to_string(), self.theme.accent),
        ];
        self.draw_center_overlay(out, w, h, " Help ", &lines)
    }

    fn render_palette_overlay(
        &self,
        out: &mut io::Stdout,
        w: usize,
        h: usize,
    ) -> Result<(), String> {
        let entries = self.filtered_palette_entries();
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        lines.push((
            format!("Query: {}", self.palette_query),
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
            for (i, item) in entries.iter().take(max_items).enumerate() {
                let prefix = if i == self.palette_selected { ">" } else { " " };
                let color = if i == self.palette_selected {
                    self.theme.ok
                } else if item.label.starts_with("Theme:") {
                    self.theme.accent
                } else {
                    self.theme.info
                };
                lines.push((format!("{prefix} {}", item.label), color));
            }
            if entries.len() > max_items {
                lines.push((
                    format!("… {} more entries", entries.len() - max_items),
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
        let content_h = cmp::min(lines.len(), max_inner_h.saturating_sub(2));
        let box_h = content_h + 2;
        let x = (w.saturating_sub(box_w)) / 2;
        let y = (h.saturating_sub(box_h)) / 2;

        let mut top = format!("┌{}┐", "─".repeat(inner_w));
        let title_text = truncate_display(title, inner_w.saturating_sub(2));
        let title_w = UnicodeWidthStr::width(title_text.as_str());
        let title_x = 1 + (inner_w.saturating_sub(title_w)) / 2;
        if title_x + title_w <= top.len() {
            let replace_start = title_x;
            let replace_end = replace_start + title_w;
            top.replace_range(replace_start..replace_end, &title_text);
        }
        draw_at(
            out,
            x as u16,
            y as u16,
            &self.style.paint(self.theme.accent, top),
        )?;

        for (i, (line, color)) in lines.iter().take(content_h).enumerate() {
            let padded = pad_display(line, inner_w);
            let painted = self.style.paint(*color, padded);
            draw_at(
                out,
                x as u16,
                (y + 1 + i) as u16,
                &format!(
                    "{}{}{}",
                    self.style.paint(self.theme.accent, "│"),
                    painted,
                    self.style.paint(self.theme.accent, "│")
                ),
            )?;
        }

        let bottom = format!("└{}┘", "─".repeat(inner_w));
        draw_at(
            out,
            x as u16,
            (y + box_h - 1) as u16,
            &self.style.paint(self.theme.accent, bottom),
        )?;
        Ok(())
    }
}

fn prev_pane(pane: Pane) -> Pane {
    match pane {
        Pane::Files => Pane::Diff,
        Pane::Log => Pane::Files,
        Pane::Diff => Pane::Log,
    }
}

fn next_pane(pane: Pane) -> Pane {
    match pane {
        Pane::Files => Pane::Log,
        Pane::Log => Pane::Diff,
        Pane::Diff => Pane::Files,
    }
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

fn git_capture(args: &[&str]) -> Result<String, String> {
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

fn git_status(args: &[&str]) -> Result<(), String> {
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
