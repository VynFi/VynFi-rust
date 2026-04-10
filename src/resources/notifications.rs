use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Parameters for listing notifications.
#[derive(Debug, Default)]
pub struct ListNotificationsParams {
    /// Filter to unread notifications only.
    pub unread: Option<bool>,
    /// Maximum number of notifications to return.
    pub limit: Option<i64>,
}

/// Notifications resource — list and manage notifications.
pub struct Notifications<'a> {
    client: &'a Client,
}

impl<'a> Notifications<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List notifications with optional filtering.
    pub async fn list(
        &self,
        params: &ListNotificationsParams,
    ) -> Result<Vec<Notification>, VynFiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(u) = params.unread {
            query.push(("unread", u.to_string()));
        }
        if let Some(l) = params.limit {
            query.push(("limit", l.to_string()));
        }
        self.client
            .request_with_params(Method::GET, "/v1/notifications", &query)
            .await
    }

    /// Mark notifications as read. Specify individual IDs or set `all` to mark
    /// all notifications.
    pub async fn mark_read(&self, req: &MarkReadRequest) -> Result<(), VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/notifications/read", Some(req))
            .await
    }
}
