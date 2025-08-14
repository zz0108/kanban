use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::task::Task;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub title: String,
    pub tasks: Vec<Task>,
}

impl Column {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, task_id: Uuid) -> Option<Task> {
        if let Some(index) = self.tasks.iter().position(|task| task.id == task_id) {
            Some(self.tasks.remove(index))
        } else {
            None
        }
    }

    pub fn get_task_mut(&mut self, task_id: Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == task_id)
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == task_id)
    }

}