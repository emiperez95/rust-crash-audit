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

    // Check each deleted file against the open issues set
    let mut out_of_sync = Vec::new();
    let mut synced = Vec::new();

    println!("Checking deleted files against open issues...");
    for file in &deleted_files {
        if open_issues.contains(&file.issue_number) {
            // Issue is still open - this is out of sync!
            if args.verbose {
                println!("  ⚠️  Issue #{} is still OPEN", file.issue_number);
            }
            out_of_sync.push(file.clone());
        } else {
            // Issue is closed or doesn't exist - this is expected
            if args.verbose {
                println!("  ✅ Issue #{} is closed", file.issue_number);
            }
            synced.push(file.clone());
        }
    }

    println!();

    // Generate report
    report::print_report(&out_of_sync, &synced, open_issues.len());

    Ok(())
}
