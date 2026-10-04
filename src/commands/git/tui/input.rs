use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;

impl App {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<KeyAction, String> {
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
            return Ok(KeyAction::Redraw);
        }

        let mut changed = false;
        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(KeyAction::Quit),
            KeyCode::Char('?') => {
                self.overlay = Some(Overlay::Help);
                changed = true;
            }
            KeyCode::Char('P') => {
                self.open_palette();
                changed = true;
            }
            KeyCode::Char('1') => {
                if self.tab != Tab::Workspace {
                    self.tab = Tab::Workspace;
                    changed = true;
                }
            }
            KeyCode::Char('2') => {
                if self.tab != Tab::Graph {
                    self.tab = Tab::Graph;
                    changed = true;
                }
            }
            KeyCode::Char('3') => {
                if self.tab != Tab::DiffViewer {
                    self.tab = Tab::DiffViewer;
                    changed = true;
                }
            }
            KeyCode::Char('5') => {
                if self.tab != Tab::Stashes {
                    self.tab = Tab::Stashes;
                    changed = true;
                }
            }
            KeyCode::Char('6') => {
                self.open_reset_picker()?;
                changed = true;
            }
            KeyCode::Char('4') => {
                if self.tab != Tab::Conflicts {
                    self.open_conflicts_tab();
                    changed = true;
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.refresh()?;
                self.status_msg = "Refreshed".to_string();
                changed = true;
            }
            KeyCode::Char('r') => {
                self.refresh()?;
                self.status_msg = "Refreshed".to_string();
                changed = true;
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                changed = true;
            }
            KeyCode::Char('D') => {
                if self.tab == Tab::Workspace {
                    self.toggle_diff_mode();
                }
                changed = true;
            }
            KeyCode::Char('d') if self.tab == Tab::DiffViewer => {
                self.toggle_detail_diff_mode();
                changed = true;
            }
            KeyCode::Char('h') | KeyCode::Left if self.tab == Tab::Workspace => {
                let before = self.pane;
                self.pane = prev_pane(self.pane);
                changed = self.pane != before;
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab if self.tab == Tab::Workspace => {
                let before = self.pane;
                self.pane = next_pane(self.pane);
                changed = self.pane != before;
            }
            KeyCode::Char('h') | KeyCode::Left if self.tab == Tab::DiffViewer => {
                self.detail_pane = DetailPane::Files;
                changed = true;
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab if self.tab == Tab::DiffViewer => {
                self.detail_pane = DetailPane::Diff;
                changed = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let before = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                );
                self.move_down_active();
                changed = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                ) != before;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let before = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                );
                self.move_up_active();
                changed = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                ) != before;
            }
            KeyCode::Char('g') => {
                let before = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                );
                self.move_home_active();
                changed = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                ) != before;
            }
            KeyCode::Char('G') => {
                let before = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                );
                self.move_end_active();
                changed = (
                    self.selected,
                    self.log_selected,
                    self.workspace_diff.scroll,
                    self.detail_diff.scroll,
                ) != before;
            }
            KeyCode::Char('s') => {
                self.stage_selected()?;
                changed = true;
            }
            KeyCode::Char('u') => {
                self.unstage_selected()?;
                changed = true;
            }
            KeyCode::Char('A') => {
                self.stage_all()?;
                changed = true;
            }
            KeyCode::Char('U') => {
                self.unstage_all()?;
                changed = true;
            }
            KeyCode::Char('c') => {
                self.input_mode = InputMode::Commit;
                self.input.clear();
                changed = true;
            }
            KeyCode::Char('p') => {
                if self.tab == Tab::Stashes {
                    self.apply_selected_stash(true)?;
                } else {
                    self.push_current_branch()?;
                }
                changed = true;
            }
            KeyCode::Char('a') if self.tab == Tab::Stashes => {
                self.apply_selected_stash(false)?;
                changed = true;
            }
            KeyCode::Char('z') => {
                self.input_mode = InputMode::StashMessage;
                self.input.clear();
                changed = true;
            }
            KeyCode::Char('S') => {
                self.open_squash_picker()?;
                changed = true;
            }
            KeyCode::Char('f') => {
                self.fetch_all()?;
                changed = true;
            }
            KeyCode::Char('L') => {
                self.pull_current_branch()?;
                changed = true;
            }
            KeyCode::Char('R') => {
                self.input_mode = InputMode::PushRemote;
                self.input = if let Some(up) = &self.upstream {
                    up.replace('/', " ")
                } else {
                    format!("origin {}", self.branch)
                };
                changed = true;
            }
            KeyCode::Char('b') => {
                self.input_mode = InputMode::NewBranch;
                self.input.clear();
                changed = true;
            }
            KeyCode::Char('i') if self.tab == Tab::Workspace && self.pane == Pane::Files => {
                self.add_selected_to_gitignore()?;
                changed = true;
            }
            KeyCode::Char('I') if self.tab == Tab::Workspace && self.pane == Pane::Files => {
                self.input_mode = InputMode::Gitignore;
                self.input.clear();
                changed = true;
            }
            KeyCode::Char('B') => {
                self.open_branch_picker()?;
                changed = true;
            }
            KeyCode::Char('m') => {
                if self.tab == Tab::Conflicts {
                    self.mark_conflicts_resolved_selected_or_marked()?;
                } else {
                    self.open_conflict_resolver();
                }
                changed = true;
            }
            KeyCode::Char('o') if self.tab == Tab::Conflicts => {
                self.resolve_conflicts_ours_selected_or_marked()?;
                changed = true;
            }
            KeyCode::Char('i') if self.tab == Tab::Conflicts => {
                self.resolve_conflicts_theirs_selected_or_marked()?;
                changed = true;
            }
            KeyCode::Char('O') if self.tab == Tab::Conflicts => {
                self.resolve_conflicts_all_ours()?;
                changed = true;
            }
            KeyCode::Char('I') if self.tab == Tab::Conflicts => {
                self.resolve_conflicts_all_theirs()?;
                changed = true;
            }
            KeyCode::Char('x') if self.tab == Tab::Conflicts => {
                self.abort_merge()?;
                changed = true;
            }
            KeyCode::Char('a') if self.tab == Tab::Conflicts => {
                self.mark_all_conflicts();
                changed = true;
            }
            KeyCode::Char(' ') if self.tab == Tab::Conflicts => {
                self.toggle_mark_conflict();
                changed = true;
            }
            KeyCode::Char(':') => {
                self.input_mode = InputMode::Command;
                self.input.clear();
                changed = true;
            }
            _ => {}
        }

        Ok(if changed {
            KeyAction::Redraw
        } else {
            KeyAction::None
        })
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<KeyAction, String> {
        let mut changed = false;
        match self.overlay {
            Some(Overlay::Help) => match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter | KeyCode::Char('q') => {
                    self.overlay = None;
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::Push) => match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.overlay = None;
                    self.push_overlay_lines.clear();
                    self.push_overlay_ok = None;
                    self.action_overlay_title = "Push".to_string();
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::BranchPicker) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    changed = true;
                }
                KeyCode::Enter => {
                    self.run_selected_branch_pick()?;
                    changed = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_branch_pick();
                    changed = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_branch_pick();
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::ConflictResolver) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    changed = true;
                }
                KeyCode::Enter => {
                    self.run_conflict_resolution_selected()?;
                    changed = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_conflict_pick();
                    changed = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_conflict_pick();
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::ResetPicker) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    changed = true;
                }
                KeyCode::Enter => {
                    self.prepare_reset();
                    changed = true;
                }
                KeyCode::Char('s') => {
                    self.reset_hard = false;
                    changed = true;
                }
                KeyCode::Char('h') => {
                    self.reset_hard = true;
                    changed = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_history();
                    changed = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_history();
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::SquashPicker) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    self.squash_marked.clear();
                    changed = true;
                }
                KeyCode::Char(' ') => {
                    self.toggle_squash_mark();
                    changed = true;
                }
                KeyCode::Enter => {
                    self.overlay = None;
                    self.input_mode = InputMode::SquashMessage;
                    self.input.clear();
                    changed = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_history();
                    changed = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_history();
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::Confirm) => match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.overlay = None;
                    self.pending_action = None;
                    changed = true;
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.execute_pending_action()?;
                    changed = true;
                }
                _ => {}
            },
            Some(Overlay::Palette) => match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    changed = true;
                }
                KeyCode::Enter => {
                    let quit = self.execute_palette_selection()?;
                    return Ok(if quit {
                        KeyAction::Quit
                    } else {
                        KeyAction::Redraw
                    });
                }
                KeyCode::Backspace => {
                    let had_query = self.palette_query.pop().is_some();
                    if had_query {
                        self.clamp_palette_selected();
                    } else {
                        self.select_prev_palette();
                    }
                    changed = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_palette();
                    changed = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_palette();
                    changed = true;
                }
                KeyCode::Char(ch) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.palette_query.push(ch);
                        self.clamp_palette_selected();
                        changed = true;
                    }
                }
                _ => {}
            },
            None => {}
        }
        Ok(if changed {
            KeyAction::Redraw
        } else {
            KeyAction::None
        })
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<KeyAction, String> {
        let mut changed = false;
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.input.clear();
                changed = true;
            }
            KeyCode::Backspace => {
                changed = self.input.pop().is_some();
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
                    InputMode::StashMessage => {
                        self.create_stash(&input)?;
                    }
                    InputMode::Gitignore => {
                        self.add_to_gitignore(&input)?;
                    }
                    InputMode::SquashMessage => {
                        self.prepare_squash(&input)?;
                    }
                    InputMode::Command => {
                        let quit = self.run_command(&input)?;
                        if quit {
                            return Ok(KeyAction::Quit);
                        }
                    }
                    InputMode::None => {}
                }
                self.input_mode = InputMode::None;
                self.input.clear();
                changed = true;
            }
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.push(ch);
                    changed = true;
                }
            }
            _ => {}
        }
        Ok(if changed {
            KeyAction::Redraw
        } else {
            KeyAction::None
        })
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
