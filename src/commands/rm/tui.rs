use std::collections::HashSet;
use std::io::{self, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    self, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use crate::core::style::Style;
use crate::core::theme;

use super::trash::{self, TrashItem};

pub fn run() -> Result<(), String> {
    let mut app = App::new();
    app.reload()?;

    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)
        .map_err(|e| e.to_string())?;

    let run_result = run_loop(&mut app);

    let _ = execute!(
        io::stdout(),
        Show,
        DisableMouseCapture,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();

    run_result
}

fn run_loop(app: &mut App) -> Result<(), String> {
    let mut dirty = true;
    loop {
        if dirty {
            app.render()?;
            dirty = false;
        }

        if !event::poll(Duration::from_millis(120)).map_err(|e| e.to_string())? {
            continue;
        }

        match event::read().map_err(|e| e.to_string())? {
            Event::Resize(_, _) => dirty = true,
            Event::Mouse(m) => {
                use crossterm::event::MouseEventKind;
                match m.kind {
                    MouseEventKind::ScrollUp => {
                        app.move_up();
                        dirty = true;
                    }
                    MouseEventKind::ScrollDown => {
                        app.move_down();
                        dirty = true;
                    }
                    _ => {}
                }
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    app.move_down();
                    dirty = true;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.move_up();
                    dirty = true;
                }
                KeyCode::Char('g') => {
                    app.selected = 0;
                    dirty = true;
                }
                KeyCode::Char('G') => {
                    app.selected = app.items.len().saturating_sub(1);
                    dirty = true;
                }
                KeyCode::Char(' ') => {
                    app.toggle_mark();
                    dirty = true;
                }
                KeyCode::Char('r') => {
                    app.restore_selected()?;
                    dirty = true;
                }
                KeyCode::Char('d') => {
                    app.purge_selected()?;
                    dirty = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

struct App {
    style: Style,
    theme: theme::Theme,
    items: Vec<TrashItem>,
    selected: usize,
    marked: HashSet<String>,
    status: String,
}

impl App {
    fn new() -> Self {
        Self {
            style: Style::for_stdout(),
            theme: theme::active(None),
            items: Vec::new(),
            selected: 0,
            marked: HashSet::new(),
            status: String::new(),
        }
    }

    fn reload(&mut self) -> Result<(), String> {
        self.items = trash::list_trash()?;
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        Ok(())
    }

    fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    fn toggle_mark(&mut self) {
        let Some(item) = self.items.get(self.selected) else {
            return;
        };
        if !self.marked.insert(item.id.clone()) {
            self.marked.remove(&item.id);
        }
    }

    fn active_indices(&self) -> Vec<usize> {
        if self.marked.is_empty() {
            if self.items.get(self.selected).is_some() {
                return vec![self.selected];
            }
            return Vec::new();
        }

        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| self.marked.contains(&item.id))
            .map(|(idx, _)| idx)
            .collect()
    }

    fn restore_selected(&mut self) -> Result<(), String> {
        let indices = self.active_indices();
        if indices.is_empty() {
            self.status = "nothing selected".to_string();
            return Ok(());
        }

        let mut count = 0usize;
        for idx in indices.into_iter().rev() {
            if let Some(item) = self.items.get(idx).cloned() {
                match trash::restore(&item) {
                    Ok(restored) => {
                        self.status = format!("restored {}", restored.display());
                        self.marked.remove(&item.id);
                        count += 1;
                    }
                    Err(err) => {
                        self.status = err;
                    }
                }
            }
        }

        self.reload()?;
        if count > 1 {
            self.status = format!("restored {count} entries");
        }
        Ok(())
    }

    fn purge_selected(&mut self) -> Result<(), String> {
        let indices = self.active_indices();
        if indices.is_empty() {
            self.status = "nothing selected".to_string();
            return Ok(());
        }

        let mut count = 0usize;
        for idx in indices.into_iter().rev() {
            if let Some(item) = self.items.get(idx).cloned() {
                match trash::purge(&item) {
                    Ok(()) => {
                        self.marked.remove(&item.id);
                        count += 1;
                    }
                    Err(err) => {
                        self.status = err;
                    }
                }
            }
        }

        self.reload()?;
        self.status = format!("deleted {count} entries from trash");
        Ok(())
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
            Clear(ClearType::All),
            Print(self.style.paint(
                self.theme.title,
                pad(&format!("dusk rm trash scanner  ({})", self.items.len()), w)
            )),
            MoveTo(0, 1),
            Print(self.style.paint(
                self.theme.subtle,
                pad("j/k move  space mark  r restore  d delete  q quit", w)
            )),
            MoveTo(0, 2),
            Print(self.style.paint(self.theme.accent, "─".repeat(w)))
        )
        .map_err(|e| e.to_string())?;

        let rows = h.saturating_sub(4);
        if self.items.is_empty() {
            queue!(
                out,
                MoveTo(0, 3),
                Print(self.style.paint(self.theme.info, pad("trash is empty", w)))
            )
            .map_err(|e| e.to_string())?;
        } else {
            let start = self.selected.saturating_sub(rows.saturating_sub(1));
            for (screen_row, item) in self.items.iter().skip(start).take(rows).enumerate() {
                let idx = start + screen_row;
                let mark = if self.marked.contains(&item.id) {
                    "*"
                } else {
                    " "
                };
                let line = format!(
                    "{} {:<18} {:<22} {}",
                    mark,
                    trim(&item.id, 18),
                    trim(&item.name, 22),
                    trim(
                        &item.original_path.display().to_string(),
                        w.saturating_sub(46)
                    )
                );
                let painted = if idx == self.selected {
                    self.style.paint("\x1b[1;97;44m", pad(&line, w))
                } else {
                    self.style.paint(self.theme.info, pad(&line, w))
                };
                queue!(out, MoveTo(0, (3 + screen_row) as u16), Print(painted))
                    .map_err(|e| e.to_string())?;
            }
        }

        queue!(
            out,
            MoveTo(0, h.saturating_sub(1) as u16),
            Print(self.style.paint(
                self.theme.ok,
                pad(
                    if self.status.is_empty() {
                        "Ready"
                    } else {
                        self.status.as_str()
                    },
                    w,
                )
            )),
            EndSynchronizedUpdate
        )
        .map_err(|e| e.to_string())?;

        out.flush().map_err(|e| e.to_string())
    }
}

fn trim(s: &str, w: usize) -> String {
    if w == 0 {
        return String::new();
    }
    let chars = s.chars().collect::<Vec<_>>();
    if chars.len() <= w {
        return s.to_string();
    }
    if w <= 1 {
        return "…".to_string();
    }
    chars[..(w - 1)].iter().collect::<String>() + "…"
}

fn pad(s: &str, w: usize) -> String {
    let mut out = trim(s, w);
    let current = out.chars().count();
    if current < w {
        out.push_str(&" ".repeat(w - current));
    }
    out
}
