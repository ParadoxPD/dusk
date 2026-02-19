use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use crate::core::icons;
use crate::core::style::Style;
use crate::core::theme::{self, THEMES, Theme};

mod actions;
mod input;
mod render;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Files,
    Log,
    Diff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffMode {
    SelectedFile,
    Repo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Workspace,
    Graph,
    CommitDiff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InputMode {
    None,
    Commit,
    NewBranch,
    SwitchBranch,
    PushRemote,
    Command,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    Help,
    Palette,
    Push,
}

enum KeyAction {
    None,
    Redraw,
    Quit,
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
    diff_mode: DiffMode,
    tab: Tab,
    input_mode: InputMode,
    input: String,
    overlay: Option<Overlay>,
    palette_query: String,
    palette_selected: usize,
    status_msg: String,
    branch: String,
    upstream: Option<String>,
    files: Vec<FileStatus>,
    selected: usize,
    log_lines: Vec<String>,
    log_commits: Vec<Option<String>>,
    log_selected: usize,
    selected_commit: Option<String>,
    diff_lines: Vec<String>,
    diff_rendered: Vec<String>,
    diff_render_width: usize,
    diff_scroll: usize,
    diff_view_rows: usize,
    commit_diff_lines: Vec<String>,
    commit_diff_rendered: Vec<String>,
    commit_diff_render_width: usize,
    commit_diff_scroll: usize,
    commit_diff_view_rows: usize,
    push_overlay_lines: Vec<String>,
    push_overlay_ok: Option<bool>,
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|e| format!("failed to enable raw mode: {e}"))?;
        execute!(io::stdout(), EnterAlternateScreen, Hide, EnableMouseCapture)
            .map_err(|e| format!("failed to enter alternate screen: {e}"))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run(theme_name: Option<&str>) -> Result<(), String> {
    let mut app = App::new(theme::active(theme_name), Style::for_stdout());
    app.refresh()?;

    let _guard = TerminalGuard::enter()?;
    let mut dirty = true;
    let mut last_cursor_phase = blink_phase();

    loop {
        if dirty {
            app.render(last_cursor_phase)?;
            dirty = false;
        }

        if !event::poll(Duration::from_millis(120)).map_err(|e| e.to_string())? {
            let phase = blink_phase();
            if phase != last_cursor_phase && app.needs_cursor_blink() {
                last_cursor_phase = phase;
                dirty = true;
            }
            continue;
        }

        match event::read().map_err(|e| e.to_string())? {
            Event::Key(key) => {
                if app.overlay.is_none()
                    && app.input_mode == InputMode::None
                    && is_diff_scroll_context(&app)
                    && is_scroll_key(&key)
                {
                    let mut delta = if is_scroll_down_key(&key) { 1 } else { -1 };
                    if drain_key_scroll_delta(
                        &mut app,
                        &mut delta,
                        &mut dirty,
                        &mut last_cursor_phase,
                    )? {
                        break;
                    }
                    if app.scroll_active_by(delta) {
                        dirty = true;
                    }
                    continue;
                }

                match app.handle_key(key)? {
                    KeyAction::Quit => break,
                    KeyAction::Redraw => {
                        last_cursor_phase = blink_phase();
                        dirty = true;
                    }
                    KeyAction::None => {}
                }
            }
            Event::Resize(_, _) => dirty = true,
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    let mut delta: isize = -1;
                    if drain_scroll_delta(&mut app, &mut delta, &mut dirty, &mut last_cursor_phase)?
                    {
                        break;
                    }
                    if app.scroll_active_by(delta) {
                        dirty = true;
                    }
                }
                MouseEventKind::ScrollDown => {
                    let mut delta: isize = 1;
                    if drain_scroll_delta(&mut app, &mut delta, &mut dirty, &mut last_cursor_phase)?
                    {
                        break;
                    }
                    if app.scroll_active_by(delta) {
                        dirty = true;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn blink_phase() -> bool {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    (ms / 500) % 2 == 0
}

fn is_diff_scroll_context(app: &App) -> bool {
    (app.tab == Tab::Workspace && app.pane == Pane::Diff) || app.tab == Tab::CommitDiff
}

fn is_scroll_down_key(key: &crossterm::event::KeyEvent) -> bool {
    matches!(
        key.code,
        crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j')
    ) && key.modifiers.is_empty()
}

fn is_scroll_up_key(key: &crossterm::event::KeyEvent) -> bool {
    matches!(
        key.code,
        crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k')
    ) && key.modifiers.is_empty()
}

fn is_scroll_key(key: &crossterm::event::KeyEvent) -> bool {
    is_scroll_down_key(key) || is_scroll_up_key(key)
}

fn drain_scroll_delta(
    app: &mut App,
    delta: &mut isize,
    dirty: &mut bool,
    last_cursor_phase: &mut bool,
) -> Result<bool, String> {
    while event::poll(Duration::from_millis(0)).map_err(|e| e.to_string())? {
        match event::read().map_err(|e| e.to_string())? {
            Event::Mouse(m2) => match m2.kind {
                MouseEventKind::ScrollUp => *delta -= 1,
                MouseEventKind::ScrollDown => *delta += 1,
                _ => {}
            },
            Event::Key(k2) => match app.handle_key(k2)? {
                KeyAction::Quit => return Ok(true),
                KeyAction::Redraw => {
                    *last_cursor_phase = blink_phase();
                    *dirty = true;
                }
                KeyAction::None => {}
            },
            Event::Resize(_, _) => *dirty = true,
            _ => {}
        }
    }
    Ok(false)
}

fn drain_key_scroll_delta(
    app: &mut App,
    delta: &mut isize,
    dirty: &mut bool,
    last_cursor_phase: &mut bool,
) -> Result<bool, String> {
    while event::poll(Duration::from_millis(0)).map_err(|e| e.to_string())? {
        match event::read().map_err(|e| e.to_string())? {
            Event::Key(k2) => {
                if is_scroll_down_key(&k2) {
                    *delta += 1;
                    continue;
                }
                if is_scroll_up_key(&k2) {
                    *delta -= 1;
                    continue;
                }
                match app.handle_key(k2)? {
                    KeyAction::Quit => return Ok(true),
                    KeyAction::Redraw => {
                        *last_cursor_phase = blink_phase();
                        *dirty = true;
                    }
                    KeyAction::None => {}
                }
            }
            Event::Mouse(m2) => match m2.kind {
                MouseEventKind::ScrollUp => *delta -= 1,
                MouseEventKind::ScrollDown => *delta += 1,
                _ => {}
            },
            Event::Resize(_, _) => *dirty = true,
            _ => {}
        }
    }
    // Keep keyboard hold movement smooth and constant speed:
    // collapse bursty repeat queues into a fixed step direction.
    let dir = delta.signum();
    *delta = if dir == 0 { 0 } else { dir * 2 };
    Ok(false)
}
