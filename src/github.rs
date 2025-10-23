use anyhow::{Context, Result};
use octocrab::Octocrab;
use std::collections::HashSet;

/// Fetch all open issues from rust-lang/rust repository
/// Returns a HashSet of issue numbers for O(1) lookup
pub async fn fetch_all_open_issues(
    github_token: Option<String>,
    verbose: bool,
) -> Result<HashSet<u64>> {
    // Build octocrab client with optional authentication
    let octocrab = if let Some(token) = github_token {
        Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to build authenticated GitHub client")?
    } else {
        if verbose {
            println!("Note: Using unauthenticated API (60 requests/hour limit)");
            println!("Set GITHUB_TOKEN environment variable for higher limits (5,000 requests/hour)");
            println!();
        }
        Octocrab::builder()
            .build()
            .context("Failed to build GitHub client")?
    };

    let mut open_issue_numbers = HashSet::new();
    let mut page_count = 0u32;

    if verbose {
        println!("Fetching open issues from rust-lang/rust...");
    }

    // Use paginate_stream for cursor-based pagination
    let mut issues_stream = octocrab
        .issues("rust-lang", "rust")
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await
        .context("Failed to start fetching open issues")?;

    loop {
        let page_items = issues_stream.items.len();

        // Add issue numbers to our set
        for issue in &issues_stream.items {
            open_issue_numbers.insert(issue.number);
        }

        page_count += 1;

        if verbose {
            println!(
                "  Fetched page {} ({} issues, {} total so far)",
                page_count,
                page_items,
                open_issue_numbers.len()
            );
        }

        // Get next page using cursor-based pagination
        match octocrab.get_page(&issues_stream.next).await {
            Ok(Some(next_page)) => {
                issues_stream = next_page;
            }
            Ok(None) => break, // No more pages
            Err(e) => {
                return Err(e).context(format!("Failed to fetch open issues (page {})", page_count + 1));
            }
        }
    }

    if verbose {
        println!(
            "\nFetched {} open issues in {} pages\n",
            open_issue_numbers.len(),
            page_count
        );
    }

    Ok(open_issue_numbers)
}
