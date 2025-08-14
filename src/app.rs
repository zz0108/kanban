use uuid::Uuid;

use crate::models::{Board, Priority, Task};

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
    AddingTask,
    MovingTask,
}

#[derive(Clone)]
pub struct EditState {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub selected_field: EditField,
}

#[derive(Clone)]
pub enum EditField {
    Title,
    Description,
    Priority,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            priority: Priority::Medium,
            selected_field: EditField::Title,
        }
    }
}

pub struct App {
    pub board: Board,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub selected_column: usize,
    pub selected_task: usize,
    pub edit_state: EditState,
    pub status_message: String,
    pub moving_task_id: Option<Uuid>,
    pub target_column: usize,
}

impl App {
    pub fn new() -> Self {
        let mut board = Board::new("My Kanban Board".to_string());
        
        let sample_task1 = Task::new("完成專案計畫".to_string())
            .with_description("規劃專案的整體架構和時程".to_string())
            .with_priority(Priority::High);
        
        let sample_task2 = Task::new("學習 Rust TUI".to_string())
            .with_description("深入了解 ratatui 框架".to_string());

        if let Some(column) = board.columns.get_mut(0) {
            column.add_task(sample_task1);
            column.add_task(sample_task2);
        }

        Self {
            board,
            input_mode: InputMode::Normal,
            should_quit: false,
            selected_column: 0,
            selected_task: 0,
            edit_state: EditState::default(),
            status_message: "Ready".to_string(),
            moving_task_id: None,
            target_column: 0,
        }
    }

    pub fn tick(&mut self) {
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn move_selection_left(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
            self.selected_task = 0;
        }
    }

    pub fn move_selection_right(&mut self) {
        if self.selected_column < self.board.columns.len().saturating_sub(1) {
            self.selected_column += 1;
            self.selected_task = 0;
        }
    }

    pub fn move_selection_up(&mut self) {
        if let Some(column) = self.board.columns.get(self.selected_column) {
            if !column.tasks.is_empty() && self.selected_task > 0 {
                self.selected_task -= 1;
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if let Some(column) = self.board.columns.get(self.selected_column) {
            if !column.tasks.is_empty() && self.selected_task < column.tasks.len().saturating_sub(1) {
                self.selected_task += 1;
            }
        }
    }

    pub fn get_selected_task_id(&self) -> Option<Uuid> {
        self.board
            .columns
            .get(self.selected_column)?
            .tasks
            .get(self.selected_task)
            .map(|task| task.id)
    }

    pub fn get_selected_column_id(&self) -> Option<Uuid> {
        self.board.columns.get(self.selected_column).map(|col| col.id)
    }

    pub fn start_adding_task(&mut self) {
        self.input_mode = InputMode::AddingTask;
        self.edit_state = EditState::default();
        self.status_message = "Enter new task title".to_string();
    }

    pub fn start_editing_task(&mut self) {
        if let Some(task_id) = self.get_selected_task_id() {
            if let Some(task) = self.board.get_task(task_id) {
                self.edit_state = EditState {
                    title: task.title.clone(),
                    description: task.description.clone().unwrap_or_default(),
                    priority: task.priority.clone(),
                    selected_field: EditField::Title,
                };
                self.input_mode = InputMode::Editing;
                self.status_message = "Editing task".to_string();
            }
        }
    }

    pub fn finish_adding_task(&mut self) {
        if !self.edit_state.title.trim().is_empty() {
            let task = Task::new(self.edit_state.title.clone())
                .with_description(if self.edit_state.description.trim().is_empty() {
                    String::new()
                } else {
                    self.edit_state.description.clone()
                })
                .with_priority(self.edit_state.priority.clone());

            if let Some(column) = self.board.columns.get_mut(self.selected_column) {
                column.add_task(task);
                self.status_message = "Task added successfully".to_string();
            }
        }
        self.input_mode = InputMode::Normal;
        self.edit_state = EditState::default();
    }

    pub fn finish_editing_task(&mut self) {
        if let Some(task_id) = self.get_selected_task_id() {
            if let Some(task) = self.board.get_task_mut(task_id) {
                task.update_title(self.edit_state.title.clone());
                task.update_description(if self.edit_state.description.trim().is_empty() {
                    None
                } else {
                    Some(self.edit_state.description.clone())
                });
                task.update_priority(self.edit_state.priority.clone());
                self.status_message = "Task updated successfully".to_string();
            }
        }
        self.input_mode = InputMode::Normal;
        self.edit_state = EditState::default();
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.edit_state = EditState::default();
        self.moving_task_id = None;
        self.status_message = "Cancelled".to_string();
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(task_id) = self.get_selected_task_id() {
            if self.board.delete_task(task_id) {
                if let Some(column) = self.board.columns.get(self.selected_column) {
                    if self.selected_task >= column.tasks.len() && !column.tasks.is_empty() {
                        self.selected_task = column.tasks.len() - 1;
                    }
                }
                self.status_message = "Task deleted successfully".to_string();
            }
        }
    }


    pub fn move_task_to_prev_column(&mut self) {
        if let Some(task_id) = self.get_selected_task_id() {
            let from_column_id = self.get_selected_column_id().unwrap();
            
            if self.selected_column > 0 {
                let to_column_id = self.board.columns[self.selected_column - 1].id;
                if self.board.move_task(task_id, from_column_id, to_column_id) {
                    self.status_message = "Task moved to previous column".to_string();
                    
                    if let Some(column) = self.board.columns.get(self.selected_column) {
                        if self.selected_task >= column.tasks.len() && !column.tasks.is_empty() {
                            self.selected_task = column.tasks.len() - 1;
                        }
                    }
                }
            }
        }
    }

    pub fn start_moving_task(&mut self) {
        if let Some(task_id) = self.get_selected_task_id() {
            self.moving_task_id = Some(task_id);
            self.target_column = self.selected_column;
            self.input_mode = InputMode::MovingTask;
            self.status_message = "Select target column (←/→ to choose, Enter to confirm, Esc to cancel)".to_string();
        }
    }

    pub fn move_target_left(&mut self) {
        if self.target_column > 0 {
            self.target_column -= 1;
        }
    }

    pub fn move_target_right(&mut self) {
        if self.target_column < self.board.columns.len().saturating_sub(1) {
            self.target_column += 1;
        }
    }

    pub fn move_edit_field_next(&mut self) {
        self.edit_state.selected_field = match self.edit_state.selected_field {
            EditField::Title => EditField::Description,
            EditField::Description => EditField::Priority,
            EditField::Priority => EditField::Title, // Cycle back to first field
        };
    }

    pub fn move_edit_field_prev(&mut self) {
        self.edit_state.selected_field = match self.edit_state.selected_field {
            EditField::Title => EditField::Priority,
            EditField::Description => EditField::Title,
            EditField::Priority => EditField::Description,
        };
    }

    

    pub fn confirm_move_task(&mut self) {
        if let Some(task_id) = self.moving_task_id {
            let from_column_id = self.get_selected_column_id().unwrap();
            let to_column_id = self.board.columns[self.target_column].id;
            
            if self.board.move_task(task_id, from_column_id, to_column_id) {
                let target_column_name = &self.board.columns[self.target_column].title;
                self.status_message = format!("Task moved to {}", target_column_name);
                
                // Adjust selection if current column lost tasks
                if let Some(column) = self.board.columns.get(self.selected_column) {
                    if self.selected_task >= column.tasks.len() && !column.tasks.is_empty() {
                        self.selected_task = column.tasks.len() - 1;
                    }
                }
            }
        }
        
        self.input_mode = InputMode::Normal;
        self.moving_task_id = None;
    }

    pub fn validate_selection(&mut self) {
        // Ensure selected_column is valid
        if self.selected_column >= self.board.columns.len() {
            self.selected_column = if self.board.columns.is_empty() { 0 } else { self.board.columns.len() - 1 };
        }
        
        // Ensure selected_task is valid for the current column
        if let Some(column) = self.board.columns.get(self.selected_column) {
            if self.selected_task >= column.tasks.len() {
                self.selected_task = if column.tasks.is_empty() { 0 } else { column.tasks.len() - 1 };
            }
        }
    }
}