use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    app::{App, EditField, InputMode},
    models::Priority,
};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    // Only handle key press events, not release events to avoid duplicate input
    if key_event.kind != KeyEventKind::Press {
        return;
    }
    
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(key_event, app),
        InputMode::AddingTask | InputMode::Editing => handle_input_mode(key_event, app),
        InputMode::MovingTask => handle_moving_mode(key_event, app),
    }
}

fn handle_normal_mode(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        
        // Navigation
        KeyCode::Char('h') | KeyCode::Left => {
            app.move_selection_left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.move_selection_right();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_selection_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_selection_up();
        }
        
        // Task operations
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.start_adding_task();
        }
        KeyCode::Enter => {
            app.start_editing_task();
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.delete_selected_task();
        }
        
        // Move task between columns
        KeyCode::Char('m') => {
            app.start_moving_task();
        }
        KeyCode::Char('M') => {
            app.move_task_to_prev_column();
        }
        
        _ => {}
    }
}

fn handle_input_mode(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Enter => {
            // Only finish adding/editing if title is not empty or just whitespace
            if !app.edit_state.title.trim().is_empty() {
                match app.input_mode {
                    InputMode::AddingTask => app.finish_adding_task(),
                    InputMode::Editing => app.finish_editing_task(),
                    _ => {}
                }
            }
        }
        KeyCode::Esc => {
            app.cancel_input();
        }
        KeyCode::Char(c) => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'c' => app.cancel_input(),
                    _ => {}
                }
            } else {
                match app.edit_state.selected_field {
                    EditField::Title => {
                        match c {
                            _ => {
                                app.edit_state.title.push(c);
                            }
                        }
                    }
                    EditField::Description => {
                        match c {
                            _ => {
                                app.edit_state.description.push(c);
                            }
                        }
                    }
                    EditField::Priority => {
                        match c {
                            '+' | '=' => {
                                app.edit_state.priority = match app.edit_state.priority {
                                    Priority::Low => Priority::Medium,
                                    Priority::Medium => Priority::High,
                                    Priority::High => Priority::Critical,
                                    Priority::Critical => Priority::Critical,
                                };
                            }
                            '-' => {
                                app.edit_state.priority = match app.edit_state.priority {
                                    Priority::Critical => Priority::High,
                                    Priority::High => Priority::Medium,
                                    Priority::Medium => Priority::Low,
                                    Priority::Low => Priority::Low,
                                };
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        KeyCode::Backspace => {
            match app.edit_state.selected_field {
                EditField::Title => {
                    app.edit_state.title.pop();
                }
                EditField::Description => {
                    app.edit_state.description.pop();
                }
                EditField::Priority => {
                    // Priority field doesn't support backspace
                }
            }
        }
        KeyCode::Tab => {
            app.move_edit_field_next();
        }
        KeyCode::Down => {
            app.move_edit_field_next();
        }
        KeyCode::Up => {
            app.move_edit_field_prev();
        }
        _ => {}
    }
}

fn handle_moving_mode(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Char('m') => {
            app.confirm_move_task();
        }
        KeyCode::Esc => {
            app.cancel_input();
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.move_target_left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.move_target_right();
        }
        _ => {}
    }
}