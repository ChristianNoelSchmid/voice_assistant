pub mod vikunja;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum TaskClientError {
    #[error("client creation failed: {0}")]
    CreateError(String),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("API returned {0}")]
    Api(reqwest::StatusCode),
}

#[async_trait]
pub trait TaskClient {
    async fn create_task(
        &self,
        title: &str,
        due_date: Option<DateTime<Utc>>,
        repeat_after: Option<i64>,
        repeat_mode: Option<i32>,
    ) -> Result<(), TaskClientError>;
}

pub type DynTaskClient = Arc<dyn TaskClient + Send + Sync>;
