use reqwest::Method;
use reqwest_eventsource::EventSource;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Parameters for listing jobs.
#[derive(Debug, Default)]
pub struct ListJobsParams {
    /// Filter by job status (e.g. `"completed"`, `"running"`).
    pub status: Option<String>,
    /// Maximum number of jobs to return (default 20, max 100).
    pub limit: Option<i64>,
    /// Offset for pagination (default 0).
    pub offset: Option<i64>,
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

    /// Submit an asynchronous generation job (legacy tables format).
    ///
    /// Returns immediately with a job ID and status links. Poll the job or use
    /// [`stream`](Self::stream) to track progress.
    pub async fn generate(&self, req: &GenerateRequest) -> Result<SubmitJobResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/generate", Some(req))
            .await
    }

    /// Submit an asynchronous generation job (config-based format).
    pub async fn generate_config(
        &self,
        req: &GenerateConfigRequest,
    ) -> Result<SubmitJobResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/generate", Some(req))
            .await
    }

    /// Submit a synchronous ("quick") generation job.
    ///
    /// Blocks server-side until the job completes (max 10,000 rows, 30s timeout).
    pub async fn generate_quick(
        &self,
        req: &GenerateRequest,
    ) -> Result<QuickJobResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/generate/quick", Some(req))
            .await
    }

    /// List jobs with optional filtering and pagination.
    pub async fn list(&self, params: &ListJobsParams) -> Result<JobList, VynFiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(ref status) = params.status {
            query.push(("status", status.clone()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset", offset.to_string()));
        }

        self.client
            .request_with_params(Method::GET, "/v1/jobs", &query)
            .await
    }

    /// Get a single job by ID.
    pub async fn get(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/jobs/{}", job_id))
            .await
    }

    /// Cancel a queued or running job.
    pub async fn cancel(&self, job_id: &str) -> Result<CancelJobResponse, VynFiError> {
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
            .request_raw(Method::GET, &format!("/v1/jobs/{}/download", job_id), &[])
            .await?;
        Ok(resp.bytes().await?)
    }

    /// Download a specific file from a completed job's output.
    pub async fn download_file(
        &self,
        job_id: &str,
        file: &str,
    ) -> Result<bytes::Bytes, VynFiError> {
        let resp = self
            .client
            .request_raw(
                Method::GET,
                &format!("/v1/jobs/{}/download/{}", job_id, file),
                &[],
            )
            .await?;
        Ok(resp.bytes().await?)
    }

    /// Download the job's full archive as raw bytes and construct a
    /// [`JobArchive`](crate::JobArchive) for ergonomic file access.
    pub async fn download_archive(&self, job_id: &str) -> Result<crate::JobArchive, VynFiError> {
        let data = self.download(job_id).await?;
        crate::JobArchive::from_bytes(&data).map_err(VynFiError::Config)
    }

    /// Download the job's full archive and write the bytes to a local file.
    pub async fn download_to(
        &self,
        job_id: &str,
        path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, VynFiError> {
        let bytes = self.download(job_id).await?;
        let p = path.as_ref().to_path_buf();
        std::fs::write(&p, &bytes).map_err(|e| VynFiError::Config(e.to_string()))?;
        Ok(p)
    }

    /// List every file in a completed job's archive with size + schema
    /// metadata. Retries on 404 up to ~4.5s to absorb managed_blob index lag.
    pub async fn list_files(&self, job_id: &str) -> Result<JobFileList, VynFiError> {
        let path = format!("/v1/jobs/{}/files", job_id);
        let mut last_err: Option<VynFiError> = None;
        for delay_ms in &[0u64, 1_500, 3_000] {
            if *delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
            }
            match self.client.request::<JobFileList>(Method::GET, &path).await {
                Ok(fl) => return Ok(fl),
                Err(e @ VynFiError::NotFound(_)) => last_err = Some(e),
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| VynFiError::Config("list_files failed".into())))
    }

    /// Get pre-built analytics for a completed job (DS 2.3+).
    pub async fn analytics(&self, job_id: &str) -> Result<JobAnalytics, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/jobs/{}/analytics", job_id))
            .await
    }

    /// Get the scheme-vs-direct fraud origin split (DS 3.1.1+).
    pub async fn fraud_split(&self, job_id: &str) -> Result<FraudSplit, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/jobs/{}/fraud-split", job_id))
            .await
    }

    /// Get aggregated audit + anomaly artifacts (VynFi API 4.1+).
    pub async fn audit_artifacts(&self, job_id: &str) -> Result<AuditArtifacts, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/jobs/{}/audit-artifacts", job_id))
            .await
    }

    /// Request an AI-suggested config tune based on the job's quality scores
    /// (DS 3.0+, Scale+).
    pub async fn tune(
        &self,
        job_id: &str,
        req: &AiTuneRequest,
    ) -> Result<AiTuneResponse, VynFiError> {
        self.client
            .request_with_body(
                Method::POST,
                &format!("/v1/jobs/{}/tune", job_id),
                Some(req),
            )
            .await
    }

    /// Poll until the job reaches a terminal state. Returns the last-seen job
    /// if the timeout is exhausted.
    pub async fn wait(
        &self,
        job_id: &str,
        poll_interval: std::time::Duration,
        timeout: std::time::Duration,
    ) -> Result<Job, VynFiError> {
        let start = std::time::Instant::now();
        loop {
            let job = self.get(job_id).await?;
            if matches!(job.status.as_str(), "completed" | "failed" | "cancelled") {
                return Ok(job);
            }
            if start.elapsed() >= timeout {
                return Ok(job);
            }
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Poll until every job in `job_ids` reaches a terminal state.
    pub async fn wait_for_many(
        &self,
        job_ids: &[String],
        poll_interval: std::time::Duration,
        timeout: std::time::Duration,
    ) -> Result<Vec<Job>, VynFiError> {
        let start = std::time::Instant::now();
        let mut results: std::collections::HashMap<String, Job> = std::collections::HashMap::new();
        let mut pending: Vec<String> = job_ids.to_vec();
        let terminal = ["completed", "failed", "cancelled"];
        while !pending.is_empty() && start.elapsed() < timeout {
            let mut still_pending = Vec::new();
            for jid in pending.drain(..) {
                match self.get(&jid).await {
                    Ok(job) => {
                        let done = terminal.contains(&job.status.as_str());
                        results.insert(jid.clone(), job);
                        if !done {
                            still_pending.push(jid);
                        }
                    }
                    Err(_) => still_pending.push(jid),
                }
            }
            pending = still_pending;
            if !pending.is_empty() {
                tokio::time::sleep(poll_interval).await;
            }
        }
        // Best-effort capture for any still-pending ids on the way out.
        for jid in &pending {
            if !results.contains_key(jid) {
                if let Ok(job) = self.get(jid).await {
                    results.insert(jid.clone(), job);
                }
            }
        }
        Ok(job_ids.iter().filter_map(|j| results.remove(j)).collect())
    }

    /// Stream NDJSON output records from a job (DS 2.3+, Scale+).
    ///
    /// Returns the raw streaming [`reqwest::Response`]; drain it with
    /// `response.bytes_stream()` and split on `\n` to get one record per line.
    /// Progress envelopes arrive as JSON objects with `"type": "_progress"`.
    pub async fn stream_ndjson(
        &self,
        job_id: &str,
        params: &NdjsonStreamParams,
    ) -> Result<reqwest::Response, VynFiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(r) = params.rate {
            query.push(("rate", r.to_string()));
        }
        if let Some(b) = params.burst {
            query.push(("burst", b.to_string()));
        }
        if let Some(p) = params.progress_interval {
            query.push(("progress_interval", p.to_string()));
        }
        if let Some(ref f) = params.file {
            query.push(("file", f.clone()));
        }
        self.client
            .request_raw(
                Method::GET,
                &format!("/v1/jobs/{}/stream/ndjson", job_id),
                &query,
            )
            .await
    }
}

/// Parameters for NDJSON output streaming.
#[derive(Debug, Default, Clone)]
pub struct NdjsonStreamParams {
    pub rate: Option<u32>,
    pub burst: Option<u32>,
    pub progress_interval: Option<u32>,
    pub file: Option<String>,
}
