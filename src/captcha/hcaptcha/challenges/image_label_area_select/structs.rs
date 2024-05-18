use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Answer<T> {
    pub task_hash: String,
    pub task_key: String,
    pub task_answer: T
}