use chrono::{Duration, NaiveDate, Utc};
use std::collections::HashSet;
use tracing::{debug, info};

use crate::api::types::VerificationType;
use crate::github::{GitHubClient, GitHubEvent};
use crate::utils::errors::AppError;

pub struct VerificationEngine {
    github_client: GitHubClient,
}

impl VerificationEngine {
    pub fn new() -> Self {
        Self {
            github_client: GitHubClient::new(),
        }
    }

    pub async fn verify_criteria(
        &self,
        events: &[GitHubEvent],
        verification_type: VerificationType,
        threshold: u32,
    ) -> Result<bool, AppError> {
        let actual_value = match verification_type {
            VerificationType::YearlyCommits => self.count_yearly_commits(events),
            VerificationType::ConsecutiveDays => self.count_consecutive_days(events),
            VerificationType::TotalStars => {
                // Extract username from events (assuming all events are from the same user)
                if let Some(event) = events.first() {
                    self.github_client
                        .count_total_stars(&event.actor.login)
                        .await?
                } else {
                    0
                }
            }
            VerificationType::PublicRepos => {
                // Extract username from events
                if let Some(event) = events.first() {
                    self.github_client
                        .count_public_repos(&event.actor.login)
                        .await?
                } else {
                    0
                }
            }
        };

        let meets_criteria = actual_value >= threshold;

        info!(
            "Verification result - Type: {:?}, Threshold: {}, Actual: {}, Meets criteria: {}",
            verification_type, threshold, actual_value, meets_criteria
        );

        Ok(meets_criteria)
    }

    fn count_yearly_commits(&self, events: &[GitHubEvent]) -> u32 {
        let one_year_ago = Utc::now() - Duration::days(365);

        // Debug: Count all events first
        let total_events = events.len();
        let push_events: Vec<_> = events
            .iter()
            .filter(|event| event.event_type == "PushEvent")
            .collect();
        let recent_push_events: Vec<_> = events
            .iter()
            .filter(|event| event.event_type == "PushEvent" && event.created_at >= one_year_ago)
            .collect();

        debug!(
            "Total events: {}, Push events: {}, Recent push events: {}",
            total_events,
            push_events.len(),
            recent_push_events.len()
        );

        let mut total_commits = 0;
        for (i, event) in recent_push_events.iter().enumerate() {
            let commits_in_event = event
                .payload
                .get("commits")
                .and_then(|commits| commits.as_array())
                .map(|commits| commits.len() as u32)
                .unwrap_or(0);

            debug!(
                "Push event {}: {} commits at {}",
                i + 1,
                commits_in_event,
                event.created_at
            );
            total_commits += commits_in_event;
        }

        info!("COMMIT COUNT BREAKDOWN - Total events: {}, Push events: {}, Recent push events: {}, Total commits: {}", 
              total_events, push_events.len(), recent_push_events.len(), total_commits);

        total_commits
    }

    fn count_consecutive_days(&self, events: &[GitHubEvent]) -> u32 {
        // Collect all unique activity dates
        let mut activity_dates = HashSet::new();

        for event in events {
            let date = event.created_at.date_naive();
            activity_dates.insert(date);
        }

        if activity_dates.is_empty() {
            return 0;
        }

        // Sort dates
        let mut sorted_dates: Vec<NaiveDate> = activity_dates.into_iter().collect();
        sorted_dates.sort();

        // Find longest consecutive streak
        let mut max_consecutive = 1;
        let mut current_consecutive = 1;

        for i in 1..sorted_dates.len() {
            let prev_date = sorted_dates[i - 1];
            let current_date = sorted_dates[i];

            if current_date == prev_date + Duration::days(1) {
                current_consecutive += 1;
            } else {
                max_consecutive = max_consecutive.max(current_consecutive);
                current_consecutive = 1;
            }
        }

        max_consecutive = max_consecutive.max(current_consecutive);

        debug!("Found {} consecutive days of activity", max_consecutive);
        max_consecutive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use serde_json::json;

    fn create_test_event(event_type: &str, days_ago: i64, commit_count: usize) -> GitHubEvent {
        let created_at = Utc::now() - Duration::days(days_ago);
        let commits = (0..commit_count)
            .map(|i| json!({"sha": format!("abc{}", i)}))
            .collect::<Vec<_>>();

        GitHubEvent {
            id: format!("event_{}", days_ago),
            event_type: event_type.to_string(),
            actor: crate::github::GitHubActor {
                id: 123,
                login: "testuser".to_string(),
            },
            repo: crate::github::GitHubRepo {
                id: 456,
                name: "testuser/testrepo".to_string(),
            },
            created_at,
            payload: json!({"commits": commits}),
        }
    }

    #[test]
    fn test_count_yearly_commits() {
        let engine = VerificationEngine::new();
        let events = vec![
            create_test_event("PushEvent", 30, 2), // 2 commits, 30 days ago
            create_test_event("PushEvent", 100, 3), // 3 commits, 100 days ago
            create_test_event("PushEvent", 400, 1), // 1 commit, 400 days ago (over 1 year)
            create_test_event("IssueEvent", 50, 0), // Not a push event
        ];

        let result = engine.count_yearly_commits(&events);
        assert_eq!(result, 5); // Only commits from within the last year
    }

    #[test]
    fn test_count_consecutive_days() {
        let engine = VerificationEngine::new();
        let events = vec![
            create_test_event("PushEvent", 1, 1), // Yesterday
            create_test_event("PushEvent", 2, 1), // 2 days ago
            create_test_event("PushEvent", 3, 1), // 3 days ago
            create_test_event("PushEvent", 5, 1), // 5 days ago (gap)
            create_test_event("PushEvent", 6, 1), // 6 days ago
        ];

        let result = engine.count_consecutive_days(&events);
        assert_eq!(result, 3); // Longest streak is 3 consecutive days
    }
}
