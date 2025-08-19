use crate::utils::errors::AppError;
use regex::Regex;

pub fn validate_github_username(username: &str) -> Result<(), AppError> {
    if username.is_empty() || username.len() > 39 {
        return Err(AppError::Validation(
            "GitHub username must be between 1 and 39 characters".to_string(),
        ));
    }

    // GitHub username pattern: alphanumeric and hyphens, but cannot start or end with hyphen
    let regex = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,37}[a-zA-Z0-9])?$")
        .expect("Invalid regex pattern");

    if !regex.is_match(username) {
        return Err(AppError::Validation(
            "Invalid GitHub username format. Must contain only alphanumeric characters and hyphens, cannot start or end with hyphen".to_string()
        ));
    }

    // Check for consecutive hyphens
    if username.contains("--") {
        return Err(AppError::Validation(
            "GitHub username cannot contain consecutive hyphens".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_usernames() {
        assert!(validate_github_username("octocat").is_ok());
        assert!(validate_github_username("test-user").is_ok());
        assert!(validate_github_username("user123").is_ok());
        assert!(validate_github_username("a").is_ok());
        assert!(validate_github_username("a-b-c").is_ok());
    }

    #[test]
    fn test_invalid_usernames() {
        assert!(validate_github_username("").is_err());
        assert!(validate_github_username("-invalid").is_err());
        assert!(validate_github_username("invalid-").is_err());
        assert!(validate_github_username("test--user").is_err());
        assert!(validate_github_username("user@invalid").is_err());
        assert!(validate_github_username(&"a".repeat(40)).is_err());
    }
}
