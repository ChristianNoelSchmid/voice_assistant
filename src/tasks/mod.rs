pub mod vikunja;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Errors that can occur when creating or communicating with a task backend.
#[derive(Debug, thiserror::Error)]
pub enum TaskClientError {
    #[error("client creation failed: {0}")]
    CreateError(String),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("API returned {0}")]
    Api(reqwest::StatusCode),
}

/// Abstraction over a task-management backend (e.g. Vikunja).
#[async_trait]
pub trait TaskClient {
    /// Create a new task with an optional due date and recurrence settings.
    ///
    /// `repeat_after` and `repeat_mode` are backend-specific; see the Vikunja
    /// implementation for their semantics.
    async fn create_task(
        &self,
        title: &str,
        due_date: Option<DateTime<Utc>>,
        repeat_after: Option<i64>,
        repeat_mode: Option<i32>,
    ) -> Result<(), TaskClientError>;
}

// Arc so a single client can be shared with command handlers without lifetime coupling.
pub type DynTaskClient = Arc<dyn TaskClient + Send + Sync>;
