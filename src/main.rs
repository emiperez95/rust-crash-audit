mod cache;
mod git;
mod github;
mod report;

use anyhow::{Context, Result};
use clap::Parser;
use chrono::NaiveDate;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rust-crash-audit",
    about = "Audit Rust repository for out-of-sync crash test files and issues",
    version
)]
struct Args {
    /// Path to the Rust repository
    #[arg(value_name = "REPO_PATH")]
    repo_path: PathBuf,

    /// Start date for scanning (format: YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    from: Option<NaiveDate>,

    /// End date for scanning (format: YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    to: Option<NaiveDate>,

    /// GitHub personal access token (or use GITHUB_TOKEN env var)
    #[arg(long, value_name = "TOKEN", env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    /// Force refresh the cache (ignore existing cache)
    #[arg(long)]
    refresh_cache: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    // Validate repository path
    if !args.repo_path.exists() {
        anyhow::bail!("Repository path does not exist: {:?}", args.repo_path);
    }

    if !args.repo_path.is_dir() {
        anyhow::bail!("Repository path is not a directory: {:?}", args.repo_path);
    }

    // Validate date range
    if let (Some(from), Some(to)) = (args.from, args.to) {
        if from > to {
            anyhow::bail!("Start date must be before end date");
        }
    }

    println!("Scanning Rust repository...");
    if let Some(from) = args.from {
        print!("Date range: {} to ", from);
        if let Some(to) = args.to {
            println!("{}", to);
        } else {
            println!("present");
        }
    }
    println!();

    // Scan git history for deleted crash test files
    let deleted_files = git::scan_deleted_crash_tests(&args.repo_path, args.from, args.to)
        .context("Failed to scan git history")?;

    println!("Found {} deleted crash test files\n", deleted_files.len());

    if deleted_files.is_empty() {
        println!("No deleted crash test files found in the specified range.");
        return Ok(());
    }

    // Load or fetch open issues (with caching)
    let open_issues = if args.refresh_cache {
        // Force refresh: fetch from API and save to cache
        if args.verbose {
            println!("Refreshing cache...\n");
        }
        let issues = github::fetch_all_open_issues(args.github_token.clone(), args.verbose)
            .await
            .context("Failed to fetch open issues from GitHub")?;

        cache::save_cache(&issues)
            .context("Failed to save cache")?;

        if !args.verbose {
            println!("Cached {} open issues\n", issues.len());
        }

        issues
    } else if cache::cache_exists() {
        // Load from cache
        let cached = cache::load_cache()
            .context("Failed to load cache")?;

        let age = cache::format_duration(cached.age());
        println!("Using cached data (updated {} ago)", age);
        println!("Use --refresh-cache to update\n");

        cached.to_hashset()
    } else {
        // No cache: fetch from API and save to cache
        let issues = github::fetch_all_open_issues(args.github_token.clone(), args.verbose)
            .await
            .context("Failed to fetch open issues from GitHub")?;

        cache::save_cache(&issues)
            .context("Failed to save cache")?;

        if !args.verbose {
            println!("Cached {} open issues\n", issues.len());
        }

        issues
    };

    // Get current crash test files to detect partial deletions
    let current_files = git::get_current_crash_test_files(&args.repo_path)
        .context("Failed to scan current crash test files")?;

    // Group deleted files by issue number
    let mut files_by_issue: std::collections::HashMap<u64, Vec<&git::DeletedCrashTest>> =
        std::collections::HashMap::new();
    for file in &deleted_files {
        files_by_issue
            .entry(file.issue_number)
            .or_insert_with(Vec::new)
            .push(file);
    }

    // Categorize issues
    let mut fully_deleted_out_of_sync = Vec::new();
    let mut fully_deleted_synced = Vec::new();
    let mut partially_deleted = Vec::new();
    let mut files_with_open_issues = 0;
    let mut files_with_closed_issues = 0;

    println!("Checking deleted files against open issues...");
    for (issue_number, files) in files_by_issue {
        // Count how many files for this issue still exist
        let remaining_count = current_files.iter().filter(|filename| {
            // Extract issue number from current filename
            if let Some(current_issue) = git::extract_issue_number_from_filename(filename) {
                current_issue == issue_number
            } else {
                false
            }
        }).count();

        // Count files for statistics
        let file_count = files.len();
        if open_issues.contains(&issue_number) {
            files_with_open_issues += file_count;
        } else {
            files_with_closed_issues += file_count;
        }

        if remaining_count > 0 {
            // Partial deletion - some files remain
            partially_deleted.push((issue_number, files, remaining_count));
            if args.verbose {
                println!(
                    "  ℹ️  Issue #{}: {} file(s) deleted, {} remain",
                    issue_number,
                    file_count,
                    remaining_count
                );
            }
        } else {
            // Full deletion - all files for this issue are gone
            if open_issues.contains(&issue_number) {
                // Issue is still open - this is out of sync!
                if args.verbose {
                    println!("  ⚠️  Issue #{} is still OPEN (all files deleted)", issue_number);
                }
                fully_deleted_out_of_sync.push((issue_number, files));
            } else {
                // Issue is closed or doesn't exist - this is expected
                if args.verbose {
                    println!("  ✅ Issue #{} is closed (all files deleted)", issue_number);
                }
                fully_deleted_synced.push((issue_number, files));
            }
        }
    }

    println!();

    // Generate report
    report::print_report(
        &fully_deleted_out_of_sync,
        &fully_deleted_synced,
        &partially_deleted,
        files_with_open_issues,
        files_with_closed_issues,
        open_issues.len(),
    );

    Ok(())
}
