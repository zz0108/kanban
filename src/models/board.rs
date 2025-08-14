use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{column::Column, task::Task};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub title: String,
    pub columns: Vec<Column>,
}

impl Board {
    pub fn new(title: String) -> Self {
        let mut board = Self {
            id: Uuid::new_v4(),
            title,
            columns: Vec::new(),
        };

        board.columns.push(Column::new("To Do".to_string()));
        board.columns.push(Column::new("In Progress".to_string()));
        board.columns.push(Column::new("Done".to_string()));

        board
    }


    pub fn get_column_mut(&mut self, column_id: Uuid) -> Option<&mut Column> {
        self.columns.iter_mut().find(|col| col.id == column_id)
    }


    pub fn move_task(&mut self, task_id: Uuid, from_column_id: Uuid, to_column_id: Uuid) -> bool {
        if from_column_id == to_column_id {
            return false;
        }

        let task = {
            if let Some(from_column) = self.get_column_mut(from_column_id) {
                if let Some(task) = from_column.remove_task(task_id) {
                    task
                } else {
                    return false;
                }
            } else {
                return false;
            }
        };

        if let Some(to_column) = self.get_column_mut(to_column_id) {
            to_column.add_task(task);
            true
        } else {
            false
        }
    }


    pub fn get_task_mut(&mut self, task_id: Uuid) -> Option<&mut Task> {
        for column in &mut self.columns {
            if let Some(task) = column.get_task_mut(task_id) {
                return Some(task);
            }
        }
        None
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        for column in &self.columns {
            if let Some(task) = column.get_task(task_id) {
                return Some(task);
            }
        }
        None
    }

    pub fn delete_task(&mut self, task_id: Uuid) -> bool {
        for column in &mut self.columns {
            if column.remove_task(task_id).is_some() {
                return true;
            }
        }
        false
    }

}