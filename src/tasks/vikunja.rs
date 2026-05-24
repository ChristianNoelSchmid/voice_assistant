use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{DynTaskClient, TaskClient, TaskClientError};

/// [`TaskClient`] backed by a Vikunja instance.
///
/// Reads connection details from environment variables via [`VikunjaClient::from_env`].
pub struct VikunjaClient {
    base_url: String,
    token: String,
    project_id: u64,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct CreateTaskBody {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_after: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_mode: Option<i32>,
}

impl VikunjaClient {
    /// Construct a [`VikunjaClient`] from `VIKUNJA_URL`, `VIKUNJA_TOKEN`, and
    /// `VIKUNJA_PROJECT_ID` environment variables.
    pub fn from_env() -> Result<DynTaskClient, TaskClientError> {
        let base_url = std::env::var("VIKUNJA_URL")
            .map_err(|_| TaskClientError::CreateError("`VIKUNJA_URL` is not set".into()))?;
        let token = std::env::var("VIKUNJA_TOKEN")
            .map_err(|_| TaskClientError::CreateError("`VIKUNJA_TOKEN` is not set".into()))?;
        let project_id = std::env::var("VIKUNJA_PROJECT_ID")
            .map_err(|_| TaskClientError::CreateError("`VIKUNJA_PROJECT_ID` is not set".into()))?
            .parse::<u64>()
            .map_err(|e| {
                TaskClientError::CreateError(format!(
                    "`VIKUNJA_PROJECT_ID` is not a valid integer: {e}"
                ))
            })?;
        Ok(Arc::new(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            project_id,
            client: reqwest::Client::new(),
        }))
    }
}

#[async_trait]
impl TaskClient for VikunjaClient {
    async fn create_task(
        &self,
        title: &str,
        due_date: Option<DateTime<Utc>>,
        repeat_after: Option<i64>,
        repeat_mode: Option<i32>,
    ) -> Result<(), TaskClientError> {
        let body = CreateTaskBody {
            title: title.to_string(),
            due_date,
            repeat_after,
            repeat_mode,
        };

        let url = format!(
            "{}/api/v1/projects/{}/tasks",
            self.base_url, self.project_id
        );
        // Vikunja's task-creation endpoint is PUT, not POST.
        let resp = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TaskClientError::Api(resp.status()));
        }

        Ok(())
    }
}
