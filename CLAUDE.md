# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains `rust-crash-audit`, a CLI tool designed to audit the rust-lang/rust repository for synchronization issues between crash test files and their associated GitHub issues.

**Purpose**: The Rust project maintains crash test files in `tests/crashes/` that are linked to GitHub issues. When an issue is fixed, the corresponding test file should be removed. This tool identifies cases where a crash test file was deleted but the associated issue remains open.

## Build and Development Commands

### Building the Tool

```bash
# Development build
cd rust-crash-audit
cargo build

# Release build (optimized)
cargo build --release
```

The release binary will be at `rust-crash-audit/target/release/rust-crash-audit`.

### Running the Tool

```bash
# Basic usage - scan a Rust repository
cargo run -- /path/to/rust-lang/rust

# With date filtering
cargo run -- /path/to/rust --from 2024-01-01

# With verbose output
cargo run -- /path/to/rust --from 2024-01-01 --verbose

# Using release build
./target/release/rust-crash-audit /path/to/rust --from 2024-01-01
```

### GitHub Authentication

The tool uses the GitHub API to fetch open issues. Provide a token via:

1. `.env` file (create from `.env.example`)
2. `GITHUB_TOKEN` environment variable
3. `--github-token` command line flag

**Note**: The token only needs read access to public repositories (no scopes required).

### Cache Management

```bash
# First run - fetches from API and caches (~2 minutes)
cargo run -- /path/to/rust --from 2024-10-15

# Subsequent runs - uses cache (instant)
cargo run -- /path/to/rust --from 2024-10-15

# Force refresh cache
cargo run -- /path/to/rust --from 2024-10-15 --refresh-cache

# Clear cache manually
rm -rf .cache/
```

### Testing

```bash
# Run unit tests
cd rust-crash-audit
cargo test

# Run tests with output
cargo test -- --nocapture
```

The tool is typically tested against a clone of the rust-lang/rust repository:

```bash
# Clone test repository (excluded from git via .gitignore)
git clone https://github.com/rust-lang/rust.git

# Test against local clone
cargo run -- ../rust --from 2024-01-01 --verbose
```

## Architecture

### Module Structure

The codebase is organized into focused modules (see `rust-crash-audit/src/`):

- **`main.rs`**: CLI argument parsing, orchestration flow, and error handling
- **`git.rs`**: Git history traversal and crash test file detection
- **`github.rs`**: GitHub API client for fetching open issues
- **`cache.rs`**: Local caching system for GitHub issue data
- **`report.rs`**: Report generation and formatting

### Core Workflow

1. **Git History Scan** (`git.rs`):
   - Walks commit history using `git2` crate
   - Uses first-parent optimization for faster traversal
   - Filters commits by date range if specified
   - Identifies deleted files matching `tests/crashes/*.rs`
   - Extracts issue numbers from filenames (e.g., `12345.rs` → issue #12345)

2. **Issue Data Loading** (`github.rs` + `cache.rs`):
   - **First run**: Fetches all open issues via GitHub API (~117 requests for ~11,631 issues)
   - **Subsequent runs**: Loads from `.cache/open_issues.json` (instant, 0 API calls)
   - Stores issues in a `HashSet<u64>` for O(1) lookup
   - Cache includes timestamp and can be manually refreshed with `--refresh-cache`

3. **Synchronization Check** (`main.rs`):
   - For each deleted crash test file:
     - Check if issue number exists in open issues set
     - If yes: Out of sync (issue should be closed or test restored)
     - If no: Properly synced (issue is already closed)

4. **Report Generation** (`report.rs`):
   - Lists out-of-sync issues with commit details
   - Shows statistics (percentage closed vs open)
   - Provides actionable recommendations

### Performance Optimizations

**Git Scanning**:
- First-parent traversal reduces commits to scan
- Path-specific diffing (`tests/crashes/*.rs` only)
- Early termination when past date range

**GitHub API**:
- Batch fetching (100 issues per page)
- Local caching eliminates redundant API calls
- ~98% reduction in API requests vs naive approach (50 vs 2,200 requests)
- Caching provides ~5-10x speedup on subsequent runs

### Key Data Structures

```rust
// Deleted crash test information
struct DeletedCrashTest {
    file_path: String,      // e.g., "tests/crashes/12345.rs"
    issue_number: u64,      // e.g., 12345
    commit_sha: String,     // Full commit hash
    commit_date: String,    // Date when deleted
}

// Cached issue data
struct CachedIssues {
    timestamp: DateTime<Utc>,
    issue_count: usize,
    issue_numbers: Vec<u64>,  // Sorted list of open issues
}
```

## Repository Structure

```
rust-repo-helpers/
├── rust-crash-audit/          # Main CLI tool
│   ├── src/
│   │   ├── main.rs           # Entry point and orchestration
│   │   ├── git.rs            # Git operations
│   │   ├── github.rs         # GitHub API client
│   │   ├── cache.rs          # Caching system
│   │   └── report.rs         # Report formatting
│   ├── Cargo.toml
│   └── README.md             # Detailed usage documentation
├── rust/                      # Test clone of rust-lang/rust (gitignored)
├── test-rust-small/          # Small test repository (gitignored)
├── APPROACHES.md             # Design considerations and alternatives
├── DEPLOYMENT_OPTIONS.md     # Deployment strategies (GitHub Actions, etc.)
└── .gitignore
```

## Important Context

- **APPROACHES.md**: Documents two approaches considered for this problem:
  1. Triagebot integration (proactive PR comments)
  2. Standalone audit tool (retrospective scanning) - **chosen approach**

- **DEPLOYMENT_OPTIONS.md**: Explores automation options for running the tool monthly:
  - GitHub Actions (recommended)
  - Triagebot integration
  - Standalone service
  - Local cron job
  - Manual execution

## Dependencies

Key crates used:
- `clap`: CLI argument parsing with derive macros
- `git2`: Git repository operations
- `octocrab`: GitHub API client
- `tokio`: Async runtime
- `chrono`: Date/time handling
- `serde`/`serde_json`: Serialization for caching
- `anyhow`: Error handling

## Known Testing Approach

The tool is designed to be run against a clone of the rust-lang/rust repository. The `/rust` and `/test-rust-small` directories are gitignored and used for local testing. Clone the repository locally to test functionality:

```bash
git clone https://github.com/rust-lang/rust.git
cargo run -- ./rust --from 2024-01-01 --verbose
```
