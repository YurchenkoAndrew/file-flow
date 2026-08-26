use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupDuplicatesResponse {
    pub count: usize,
    pub freed_space: u64,
}