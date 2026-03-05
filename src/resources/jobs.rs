use reqwest::Method;
use reqwest_eventsource::EventSource;
use serde::Deserialize;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Parameters for listing jobs.
#[derive(Debug, Default)]
pub struct ListJobsParams {
    /// Maximum number of jobs to return.
    pub limit: Option<u32>,
    /// Filter by job status (e.g. `"completed"`, `"running"`).
    pub status: Option<String>,
    /// Cursor-based pagination: return jobs after this cursor.
    pub after: Option<String>,
    /// Cursor-based pagination: return jobs before this cursor.
    pub before: Option<String>,
}

/// A parsed Server-Sent Event from the job progress stream.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// The event type (e.g. `"progress"`, `"complete"`, `"error"`).
    pub event: String,
    /// The JSON payload.
    pub data: serde_json::Value,
}

/// Intermediate struct for deserializing the paginated jobs response.
#[derive(Deserialize)]
struct JobListResponse {
    #[serde(alias = "data")]
    jobs: Vec<Job>,
    #[serde(default)]
    has_more: bool,
    next_cursor: Option<String>,
}

/// Jobs resource — submit, list, get, cancel, stream, and download generation
/// jobs.
pub struct Jobs<'a> {
    client: &'a Client,
}

impl<'a> Jobs<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Submit an asynchronous generation job.
    ///
    /// Returns immediately with a job ID and status links. Poll the job or use
    /// [`stream`](Self::stream) to track progress.
    pub async fn generate(&self, req: &GenerateRequest) -> Result<SubmitJobResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/generate", Some(req))
            .await
    }

    /// Submit a synchronous ("quick") generation job.
    ///
    /// Blocks until the job completes and returns the finished [`Job`] with
    /// its output path.
    pub async fn generate_quick(&self, req: &GenerateRequest) -> Result<Job, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/generate/quick", Some(req))
            .await
    }

    /// List jobs with optional filtering and pagination.
    pub async fn list(&self, params: &ListJobsParams) -> Result<JobList, VynFiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref status) = params.status {
            query.push(("status", status.clone()));
        }
        if let Some(ref after) = params.after {
            query.push(("after", after.clone()));
        }
        if let Some(ref before) = params.before {
            query.push(("before", before.clone()));
        }

        let resp: JobListResponse = self
            .client
            .request_with_params(Method::GET, "/v1/jobs", &query)
            .await?;

        Ok(JobList {
            jobs: resp.jobs,
            has_more: resp.has_more,
            next_cursor: resp.next_cursor,
        })
    }

    /// Get a single job by ID.
    pub async fn get(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/jobs/{}", job_id))
            .await
    }

    /// Cancel a running job.
    pub async fn cancel(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client
            .request(Method::DELETE, &format!("/v1/jobs/{}", job_id))
            .await
    }

    /// Open an SSE stream for real-time job progress updates.
    ///
    /// Returns an [`EventSource`] that implements `Stream`. Consume it with
    /// `futures::StreamExt::next()`:
    ///
    /// ```ignore
    /// use futures::StreamExt;
    /// use reqwest_eventsource::Event;
    ///
    /// let mut es = client.jobs().stream("job_123");
    /// while let Some(event) = es.next().await {
    ///     match event {
    ///         Ok(Event::Message(msg)) => println!("{}: {}", msg.event, msg.data),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn stream(&self, job_id: &str) -> EventSource {
        let url = self.client.url(&format!("/v1/jobs/{}/stream", job_id));
        let builder = self.client.http().get(&url);
        EventSource::new(builder).expect("valid request builder")
    }

    /// Download the output file of a completed job as raw bytes.
    pub async fn download(&self, job_id: &str) -> Result<bytes::Bytes, VynFiError> {
        let resp = self
            .client
            .request_raw(Method::GET, &format!("/v1/jobs/{}/download", job_id))
            .await?;
        Ok(resp.bytes().await?)
    }
}
