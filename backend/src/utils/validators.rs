use anyhow::{anyhow, Result};
use url::Url;

/// Validate that a string is a valid URL with http or https scheme
#[allow(dead_code)]
pub fn validate_url(url_str: &str) -> Result<Url> {
    let url = Url::parse(url_str)
        .map_err(|e| anyhow!("Invalid URL format: {}", e))?;

    // Only allow http and https schemes
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(anyhow!(
            "URL must use http or https scheme, got: {}",
            url.scheme()
        ));
    }

    // Must have a host
    if url.host_str().is_none() {
        return Err(anyhow!("URL must have a host"));
    }

    Ok(url)
}

/// Validate that a URL is a valid instance URL for self-hosted platforms
/// Returns the normalized base URL (no trailing slash, no path)
#[allow(dead_code)]
pub fn validate_instance_url(url_str: &str) -> Result<String> {
    let url = validate_url(url_str)?;

    // Normalize: remove trailing slash and path
    let normalized = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap(),
        url.port()
            .map(|p| format!(":{}", p))
            .unwrap_or_default()
    );

    Ok(normalized)
}

/// Validate username (alphanumeric, hyphens, underscores, 1-39 chars for GitHub compatibility)
#[allow(dead_code)]
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() || username.len() > 39 {
        return Err(anyhow!(
            "Username must be between 1 and 39 characters"
        ));
    }

    // Allow alphanumeric, hyphens, and underscores
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "Username can only contain alphanumeric characters, hyphens, and underscores"
        ));
    }

    Ok(())
}

/// Validate that a platform type is supported
pub fn validate_platform(platform: &str) -> Result<()> {
    match platform {
        "github" | "gitlab" | "gitea" => Ok(()),
        _ => Err(anyhow!(
            "Unsupported platform '{}'. Supported: github, gitlab, gitea",
            platform
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://github.com").is_ok());
        assert!(validate_url("http://localhost:3000").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_validate_instance_url() {
        assert_eq!(
            validate_instance_url("https://gitlab.com/foo/bar").unwrap(),
            "https://gitlab.com"
        );
        assert_eq!(
            validate_instance_url("https://gitlab.com/").unwrap(),
            "https://gitlab.com"
        );
        assert_eq!(
            validate_instance_url("http://localhost:3000").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("octocat").is_ok());
        assert!(validate_username("my-user_123").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username(&"a".repeat(40)).is_err());
        assert!(validate_username("user@example").is_err());
    }

    #[test]
    fn test_validate_platform() {
        assert!(validate_platform("github").is_ok());
        assert!(validate_platform("gitlab").is_ok());
        assert!(validate_platform("gitea").is_ok());
        assert!(validate_platform("bitbucket").is_err());
    }
}
