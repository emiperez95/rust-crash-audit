use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CACHE_DIR: &str = ".cache";
const CACHE_FILE: &str = "open_issues.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedIssues {
    pub timestamp: DateTime<Utc>,
    pub issue_count: usize,
    pub issue_numbers: Vec<u64>,
}

impl CachedIssues {
    pub fn to_hashset(&self) -> HashSet<u64> {
        self.issue_numbers.iter().copied().collect()
    }

    pub fn age(&self) -> Duration {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.timestamp);
        Duration::from_secs(elapsed.num_seconds().max(0) as u64)
    }
}

/// Get the cache file path
fn cache_path() -> PathBuf {
    PathBuf::from(CACHE_DIR).join(CACHE_FILE)
}

/// Check if cache exists
pub fn cache_exists() -> bool {
    cache_path().exists()
}

/// Load cached open issues from file
pub fn load_cache() -> Result<CachedIssues> {
    let path = cache_path();
    let contents = fs::read_to_string(&path)
        .context("Failed to read cache file")?;

    let cached: CachedIssues = serde_json::from_str(&contents)
        .context("Failed to parse cache file")?;

    Ok(cached)
}

/// Save open issues to cache file
pub fn save_cache(issues: &HashSet<u64>) -> Result<()> {
    // Create cache directory if it doesn't exist
    let cache_dir = Path::new(CACHE_DIR);
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)
            .context("Failed to create cache directory")?;
    }

    // Convert HashSet to Vec for serialization
    let mut issue_vec: Vec<u64> = issues.iter().copied().collect();
    issue_vec.sort(); // Sort for consistency

    let cached = CachedIssues {
        timestamp: Utc::now(),
        issue_count: issues.len(),
        issue_numbers: issue_vec,
    };

    let json = serde_json::to_string_pretty(&cached)
        .context("Failed to serialize cache")?;

    fs::write(cache_path(), json)
        .context("Failed to write cache file")?;

    Ok(())
}

/// Format duration in human-readable form
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{} second{}", secs, if secs == 1 { "" } else { "s" })
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} minute{}", mins, if mins == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = secs / 86400;
        format!("{} day{}", days, if days == 1 { "" } else { "s" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30 seconds");
        assert_eq!(format_duration(Duration::from_secs(1)), "1 second");
        assert_eq!(format_duration(Duration::from_secs(90)), "1 minute");
        assert_eq!(format_duration(Duration::from_secs(120)), "2 minutes");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1 hour");
        assert_eq!(format_duration(Duration::from_secs(7200)), "2 hours");
        assert_eq!(format_duration(Duration::from_secs(86400)), "1 day");
        assert_eq!(format_duration(Duration::from_secs(172800)), "2 days");
    }
}
