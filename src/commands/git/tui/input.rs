use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;

impl App {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        if self.input_mode != InputMode::None {
            return self.handle_input_key(key);
        }

        if (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P')))
            || matches!(key.code, KeyCode::Char('\u{10}'))
        {
            self.open_palette();
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(true),
            KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            KeyCode::Char('P') => self.open_palette(),
            KeyCode::Char('1') => self.tab = Tab::Workspace,
            KeyCode::Char('2') => self.tab = Tab::Graph,
            KeyCode::Char('3') => self.tab = Tab::CommitDiff,
            KeyCode::Char('r') => {
                self.refresh()?;
                self.status_msg = "Refreshed".to_string();
            }
            KeyCode::Char('t') => self.cycle_theme(),
            KeyCode::Char('h') | KeyCode::Left if self.tab == Tab::Workspace => {
                self.pane = prev_pane(self.pane)
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab if self.tab == Tab::Workspace => {
                self.pane = next_pane(self.pane)
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_down_active(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up_active(),
            KeyCode::Char('g') => self.move_home_active(),
            KeyCode::Char('G') => self.move_end_active(),
            KeyCode::Char('s') => self.stage_selected()?,
            KeyCode::Char('u') => self.unstage_selected()?,
            KeyCode::Char('A') => self.stage_all()?,
            KeyCode::Char('U') => self.unstage_all()?,
            KeyCode::Char('c') => {
                self.input_mode = InputMode::Commit;
                self.input.clear();
            }
            KeyCode::Char('p') => self.push_current_branch()?,
            KeyCode::Char('R') => {
                self.input_mode = InputMode::PushRemote;
                self.input = if let Some(up) = &self.upstream {
                    up.replace('/', " ")
                } else {
                    format!("origin {}", self.branch)
                };
            }
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
            self.refresh_commit_diff();
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
                KeyCode::Esc => self.overlay = None,
                KeyCode::Enter => return self.execute_palette_selection(),
                KeyCode::Backspace => {
                    let changed = self.palette_query.pop().is_some();
                    if changed {
                        self.clamp_palette_selected();
                    } else {
                        self.select_prev_palette();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => self.select_prev_palette(),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_palette(),
                KeyCode::Char(ch) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.palette_query.push(ch);
                        self.clamp_palette_selected();
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
                            super::actions::git_status(&["commit", "-m", &input])?;
                            self.status_msg = format!("Committed: {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::NewBranch => {
                        if input.is_empty() {
                            self.status_msg = "Branch name cannot be empty".to_string();
                        } else {
                            super::actions::git_status(&["switch", "-c", &input])?;
                            self.status_msg = format!("Created and switched to {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::SwitchBranch => {
                        if input.is_empty() {
                            self.status_msg = "Branch name cannot be empty".to_string();
                        } else {
                            super::actions::git_status(&["switch", &input])?;
                            self.status_msg = format!("Switched to {input}");
                            self.refresh()?;
                        }
                    }
                    InputMode::PushRemote => {
                        let spec = input.trim();
                        if spec.is_empty() {
                            self.status_msg =
                                "Usage: <remote> <branch> or <remote>/<branch>".to_string();
                        } else if let Some((remote, branch)) = spec.split_once(' ') {
                            self.push_to_remote_branch(remote.trim(), branch.trim(), true)?;
                        } else if let Some((remote, branch)) = spec.split_once('/') {
                            self.push_to_remote_branch(remote.trim(), branch.trim(), true)?;
                        } else {
                            self.status_msg =
                                "Usage: <remote> <branch> or <remote>/<branch>".to_string();
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
