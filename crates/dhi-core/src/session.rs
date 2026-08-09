use crate::error::Result;
use crate::types::EngineEvent;
use crate::types::{Task, TaskContract};
use std::collections::VecDeque;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Session {
    pub id: Uuid,
    pub project_root: PathBuf,
    pub working_directory: PathBuf,
    pub active_task: Option<Task>,
    pub active_contract: Option<TaskContract>,
    pub event_log: VecDeque<EngineEvent>,
}

impl Session {
    pub fn new(project_root: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_root,
            working_directory,
            active_task: None,
            active_contract: None,
            event_log: VecDeque::new(),
        }
    }

    pub fn start_task(&mut self, raw_input: String) -> Result<Uuid> {
        let task = Task {
            id: Uuid::new_v4(),
            raw_input,
            created_at: chrono::Utc::now(),
        };
        let task_id = task.id;
        self.active_task = Some(task);
        Ok(task_id)
    }

    pub fn set_contract(&mut self, contract: TaskContract) {
        self.active_contract = Some(contract);
    }
}
