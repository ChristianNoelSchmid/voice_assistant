use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{DynTaskClient, TaskClient, TaskClientError};

/// [`TaskClient`] backed by a Vikunja instance.
///
/// Reads connection details from environment variables via [`VikunjaClient::from_env`].
/// The project to write into is supplied per-call via [`TaskClient::create_task`].
pub struct VikunjaClient {
    base_url: String,
    token: String,
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
    /// Construct a [`VikunjaClient`] from a base URL and an API token.
    ///
    /// `base_url` is expected to be already normalised (trailing slash stripped);
    /// [`Config::load`] does this before calling constructors. The token comes
    /// from the `VIKUNJA_TOKEN` environment variable, read in `main`.
    pub fn new(base_url: String, token: String) -> DynTaskClient {
        Arc::new(Self {
            base_url,
            token,
            client: reqwest::Client::new(),
        })
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
        project_id: u64,
    ) -> Result<(), TaskClientError> {
        let body = CreateTaskBody {
            title: title.to_string(),
            due_date,
            repeat_after,
            repeat_mode,
        };

        let url = format!("{}/api/v1/projects/{}/tasks", self.base_url, project_id);
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
