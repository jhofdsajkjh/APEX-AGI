//! Swarm Agent Coordinator
//! Coordinates multiple AI agents working together on tasks.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Handle representing an agent in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHandle {
    pub id: String,
    pub name: String,
    pub status: String,
    pub capabilities: Vec<String>,
}

/// A task to be executed by the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTask {
    pub id: String,
    pub description: String,
    pub assigned_agent: Option<String>,
    pub status: String,
    pub result: Option<String>,
}

/// Coordinates agents in the swarm.
pub struct SwarmCoordinator {
    agents: Arc<RwLock<HashMap<String, AgentHandle>>>,
    tasks: Arc<RwLock<HashMap<String, SwarmTask>>>,
    task_counter: AtomicU64,
}

impl SwarmCoordinator {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_counter: AtomicU64::new(1),
        }
    }

    pub fn register_agent(&self, handle: AgentHandle) {
        self.agents.write().insert(handle.id.clone(), handle);
    }

    pub fn unregister_agent(&self, id: &str) -> bool {
        self.agents.write().remove(id).is_some()
    }

    pub fn get_agent(&self, id: &str) -> Option<AgentHandle> {
        self.agents.read().get(id).cloned()
    }

    pub fn list_agents(&self) -> Vec<AgentHandle> {
        self.agents.read().values().cloned().collect()
    }

    pub fn create_task(&self, description: &str) -> SwarmTask {
        let id = self.task_counter.fetch_add(1, Ordering::SeqCst);
        let task = SwarmTask {
            id: format!("task_{}", id),
            description: description.to_string(),
            assigned_agent: None,
            status: "pending".to_string(),
            result: None,
        };
        self.tasks.write().insert(task.id.clone(), task.clone());
        task
    }

    pub fn assign_task(&self, task_id: &str, agent_id: &str) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.assigned_agent = Some(agent_id.to_string());
            task.status = "assigned".to_string();
            true
        } else {
            false
        }
    }

    pub fn complete_task(&self, task_id: &str, result: &str) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = "completed".to_string();
            task.result = Some(result.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_task(&self, id: &str) -> Option<SwarmTask> {
        self.tasks.read().get(id).cloned()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.read().len()
    }

    pub fn task_count(&self) -> usize {
        self.tasks.read().len()
    }
}

impl Default for SwarmCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
