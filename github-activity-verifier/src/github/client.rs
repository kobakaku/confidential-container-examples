use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::github::{GitHubError, GitHubEvent, GitHubUser, GitHubUserRepo};

const GITHUB_API_BASE: &str = "https://api.github.com";
const EVENTS_PER_PAGE: u8 = 100;
const MAX_PAGES: u8 = 3;

pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("GitHub-Activity-Verifier/1.0"),
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        let token = std::env::var("GITHUB_TOKEN").ok();
        if let Some(ref token) = token {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("token {}", token)).unwrap(),
            );
            info!("GitHub token configured for enhanced rate limits");
        } else {
            warn!("No GitHub token configured - using anonymous access with lower rate limits");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, token }
    }

    pub async fn fetch_user_events(&self, username: &str) -> Result<Vec<GitHubEvent>, GitHubError> {
        let mut all_events = Vec::new();

        for page in 1..=MAX_PAGES {
            let url = format!(
                "{}/users/{}/events?per_page={}&page={}",
                GITHUB_API_BASE, username, EVENTS_PER_PAGE, page
            );

            debug!("Fetching GitHub events: {}", url);

            let response = self.client.get(&url).send().await?;
            let status = response.status();

            if status == 404 {
                return Err(GitHubError::UserNotFound(username.to_string()));
            }

            if status == 403 {
                // Check if it's rate limiting
                if let Some(rate_limit) = response.headers().get("X-RateLimit-Remaining") {
                    if rate_limit == "0" {
                        return Err(GitHubError::RateLimit);
                    }
                }
                return Err(GitHubError::ApiError {
                    status: status.as_u16(),
                    message: "Forbidden - check API token permissions".to_string(),
                });
            }

            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::ApiError {
                    status: status.as_u16(),
                    message: error_text,
                });
            }

            let events: Vec<GitHubEvent> = response.json().await?;

            if events.is_empty() {
                debug!("No more events found, stopping pagination");
                break;
            }

            debug!("Fetched {} events from page {}", events.len(), page);

            // Debug: Show event types for first page
            if page == 1 && !events.is_empty() {
                let event_types: std::collections::HashMap<String, usize> =
                    events
                        .iter()
                        .fold(std::collections::HashMap::new(), |mut acc, event| {
                            *acc.entry(event.event_type.clone()).or_insert(0) += 1;
                            acc
                        });
                debug!("Event types breakdown: {:?}", event_types);

                // Show recent events
                for (i, event) in events.iter().take(5).enumerate() {
                    debug!(
                        "Event {}: {} at {}",
                        i + 1,
                        event.event_type,
                        event.created_at
                    );
                }
            }

            all_events.extend(events);
        }

        info!(
            "Fetched total {} events for user: {}",
            all_events.len(),
            username
        );
        Ok(all_events)
    }

    pub async fn fetch_user(&self, username: &str) -> Result<GitHubUser, GitHubError> {
        let url = format!("{}/users/{}", GITHUB_API_BASE, username);

        debug!("Fetching GitHub user: {}", url);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status == 404 {
            return Err(GitHubError::UserNotFound(username.to_string()));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let user: GitHubUser = response.json().await?;
        debug!("Fetched user info for: {}", username);
        Ok(user)
    }

    pub async fn fetch_user_repos(
        &self,
        username: &str,
        page: u32,
    ) -> Result<Vec<GitHubUserRepo>, GitHubError> {
        let url = format!(
            "{}/users/{}/repos?per_page=100&page={}",
            GITHUB_API_BASE, username, page
        );

        debug!("Fetching GitHub repos: {}", url);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status == 404 {
            return Err(GitHubError::UserNotFound(username.to_string()));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let repos: Vec<GitHubUserRepo> = response.json().await?;
        debug!("Fetched {} repos from page {}", repos.len(), page);
        Ok(repos)
    }

    pub async fn count_total_stars(&self, username: &str) -> Result<u32, GitHubError> {
        let mut total_stars = 0;
        let mut page = 1;

        loop {
            let repos = self.fetch_user_repos(username, page).await?;
            if repos.is_empty() {
                break;
            }

            total_stars += repos.iter().map(|repo| repo.stargazers_count).sum::<u32>();
            page += 1;

            // Limit to 10 pages (1000 repos) to prevent excessive API calls
            if page > 10 {
                warn!(
                    "User {} has more than 1000 repos, limiting star count calculation",
                    username
                );
                break;
            }
        }

        info!(
            "User {} has {} total stars across {} pages of repos",
            username,
            total_stars,
            page - 1
        );
        Ok(total_stars)
    }

    pub async fn count_public_repos(&self, username: &str) -> Result<u32, GitHubError> {
        let user = self.fetch_user(username).await?;
        info!("User {} has {} public repos", username, user.public_repos);
        Ok(user.public_repos)
    }
}
