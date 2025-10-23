use anyhow::{Context, Result};
use chrono::NaiveDate;
use git2::{Delta, DiffOptions, Repository};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DeletedCrashTest {
    pub file_path: String,
    pub issue_number: u64,
    pub commit_sha: String,
    pub commit_date: String,
    pub pr_number: Option<u64>,
}

/// Scan git history for deleted crash test files
pub fn scan_deleted_crash_tests(
    repo_path: &Path,
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
) -> Result<Vec<DeletedCrashTest>> {
    let repo = Repository::open(repo_path)
        .context("Failed to open git repository")?;

    let mut revwalk = repo.revwalk()
        .context("Failed to create revwalk")?;

    // Optimization: Only follow first parent (main branch history)
    // This dramatically reduces commits to scan
    revwalk.simplify_first_parent()
        .context("Failed to simplify to first parent")?;

    // Start from HEAD
    revwalk.push_head()
        .context("Failed to push HEAD")?;

    let mut deleted_files = Vec::new();
    let mut commits_scanned = 0;

    // Walk through commits
    for oid in revwalk {
        let oid = oid.context("Failed to get commit OID")?;
        let commit = repo.find_commit(oid)
            .context("Failed to find commit")?;

        commits_scanned += 1;

        // Progress indicator every 1000 commits
        if commits_scanned % 1000 == 0 {
            eprint!("\r  Scanned {} commits...", commits_scanned);
        }

        // Get commit timestamp
        let commit_time = commit.time();
        let commit_timestamp = commit_time.seconds();
        let commit_date = chrono::DateTime::from_timestamp(commit_timestamp, 0)
            .context("Invalid timestamp")?
            .date_naive();

        // Apply date filtering - break early if we're past the from_date
        if let Some(from) = from_date {
            if commit_date < from {
                // We've gone past the from_date, stop scanning
                break;
            }
        }

        if let Some(to) = to_date {
            if commit_date > to {
                // Haven't reached the to_date yet, skip this commit
                continue;
            }
        }

        // Get parent commit (if exists)
        if commit.parent_count() == 0 {
            continue; // Skip initial commit
        }

        let parent = commit.parent(0)
            .context("Failed to get parent commit")?;

        let tree = commit.tree()
            .context("Failed to get commit tree")?;
        let parent_tree = parent.tree()
            .context("Failed to get parent tree")?;

        // Create diff between parent and current commit
        // Optimization: Only diff files in tests/crashes/ directory
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec("tests/crashes/*.rs");

        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut diff_opts))
            .context("Failed to create diff")?;

        // Look for deleted files in tests/crashes/
        for delta in diff.deltas() {
            if delta.status() == Delta::Deleted {
                if let Some(old_file) = delta.old_file().path() {
                    let path_str = old_file.to_string_lossy();

                    // Extract issue number from filename
                    if let Some(issue_number) = extract_issue_number(&path_str) {
                        // Extract PR number from commit message
                        let commit_message = commit.message().unwrap_or("");
                        let pr_number = extract_pr_number(commit_message);

                        deleted_files.push(DeletedCrashTest {
                            file_path: path_str.to_string(),
                            issue_number,
                            commit_sha: commit.id().to_string(),
                            commit_date: commit_date.to_string(),
                            pr_number,
                        });
                    }
                }
            }
        }
    }

    // Clear progress line
    if commits_scanned >= 1000 {
        eprintln!("\r  Scanned {} commits total", commits_scanned);
    }

    Ok(deleted_files)
}

/// Extract issue number from crash test filename
/// Examples:
/// - "tests/crashes/12345.rs" -> Some(12345)
/// - "tests/crashes/12345-foo.rs" -> Some(12345)
/// - "tests/crashes/foo.rs" -> None
fn extract_issue_number(path: &str) -> Option<u64> {
    let filename = Path::new(path)
        .file_stem()?
        .to_str()?;

    // Try to parse the entire filename as a number
    if let Ok(num) = filename.parse::<u64>() {
        return Some(num);
    }

    // Try to extract number from beginning (e.g., "12345-foo" -> 12345)
    if let Some(dash_pos) = filename.find('-') {
        if let Ok(num) = filename[..dash_pos].parse::<u64>() {
            return Some(num);
        }
    }

    None
}

/// Extract PR number from commit message
/// Rust bors commits follow the pattern: "Auto merge of #12345 - ..."
/// Examples:
/// - "Auto merge of #147900 - Zalathar:rollup-ril6jsi, r=Zalathar" -> Some(147900)
/// - "Regular commit message" -> None
fn extract_pr_number(message: &str) -> Option<u64> {
    // Look for "Auto merge of #" followed by digits
    let prefix = "Auto merge of #";
    if let Some(start) = message.find(prefix) {
        let number_start = start + prefix.len();
        let rest = &message[number_start..];

        // Extract digits until we hit a non-digit character
        let number_str: String = rest.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        return number_str.parse::<u64>().ok();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_issue_number() {
        assert_eq!(extract_issue_number("tests/crashes/12345.rs"), Some(12345));
        assert_eq!(extract_issue_number("tests/crashes/12345-foo.rs"), Some(12345));
        assert_eq!(extract_issue_number("tests/crashes/98765-bar-baz.rs"), Some(98765));
        assert_eq!(extract_issue_number("tests/crashes/foo.rs"), None);
        assert_eq!(extract_issue_number("tests/crashes/foo-12345.rs"), None);
    }

    #[test]
    fn test_extract_pr_number() {
        assert_eq!(
            extract_pr_number("Auto merge of #147900 - Zalathar:rollup-ril6jsi, r=Zalathar"),
            Some(147900)
        );
        assert_eq!(
            extract_pr_number("Auto merge of #12345 - username:branch, r=reviewer"),
            Some(12345)
        );
        assert_eq!(
            extract_pr_number("Regular commit message without PR"),
            None
        );
        assert_eq!(
            extract_pr_number("Mention #12345 but not auto merge"),
            None
        );
    }
}
