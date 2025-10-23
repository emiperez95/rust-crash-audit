# rust-crash-audit

A highly optimized CLI tool to audit the Rust repository for out-of-sync crash test files and their associated GitHub issues.

## Purpose

The Rust project maintains crash test files in `tests/crashes/` that are linked to GitHub issues. When an issue is fixed, the corresponding test file should be removed. This tool helps identify cases where:

- A crash test file was deleted, but the associated issue is still open
- This indicates the issue may need to be closed, or the test was removed prematurely

## Features

- **Optimized Performance**: Fetches all open issues once (~100 API requests) instead of checking each file individually
- Scans git history for deleted files in `tests/crashes/`
- Supports date range filtering
- Identifies out-of-sync issues with zero additional API calls
- Generates detailed reports with statistics
- Optional GitHub authentication for higher rate limits

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/rust-crash-audit`.

## Usage

### Basic Usage

```bash
rust-crash-audit /path/to/rust-lang/rust
```

### With GitHub Authentication (Recommended)

For faster execution and higher rate limits, provide a GitHub personal access token:

**Option 1: Using .env file (Easiest)**
```bash
# Copy the example file
cp .env.example .env

# Edit .env and add your token
# GITHUB_TOKEN=ghp_your_token_here

# Now just run the tool - it will automatically load from .env
rust-crash-audit /path/to/rust
```

**Option 2: Environment variable**
```bash
export GITHUB_TOKEN=ghp_your_token_here
rust-crash-audit /path/to/rust
```

**Option 3: Command line flag**
```bash
rust-crash-audit /path/to/rust --github-token ghp_your_token_here
```

**Creating a GitHub Token:**
1. Go to https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Give it a name (e.g., "rust-crash-audit")
4. **Leave all checkboxes unchecked** (read-only access to public repos)
5. Click "Generate token" and copy it

### With Date Range

```bash
# Scan from specific date to present
rust-crash-audit /path/to/rust --from 2024-01-01

# Scan specific date range
rust-crash-audit /path/to/rust --from 2024-01-01 --to 2024-12-31
```

### Verbose Output

```bash
rust-crash-audit /path/to/rust --verbose
```

### Using Cache (Faster Subsequent Runs)

The tool automatically caches open issues to `.cache/open_issues.json` to speed up subsequent runs.

**First run - fetches from GitHub API and saves cache:**
```bash
./target/release/rust-crash-audit ../rust --from 2024-10-15 --verbose
# → Fetching open issues... (~2 minutes)
# → Cached 10,000+ open issues
```

**Subsequent runs - loads from cache (instant!):**
```bash
./target/release/rust-crash-audit ../rust --from 2024-10-15 --verbose
# → Using cached data (updated 5 minutes ago)
# → Use --refresh-cache to update
# → (GitHub API section skipped - uses cache)
```

**Force refresh cache:**
```bash
./target/release/rust-crash-audit ../rust --from 2024-10-15 --refresh-cache --verbose
# → Refreshing cache...
# → Fetching open issues... (~2 minutes)
# → Cached 10,000+ open issues
```

**Clear cache manually:**
```bash
rm -rf .cache/
```

## Example Output

```
Scanning Rust repository...
Date range: 2024-01-01 to present

Found 245 deleted crash test files

Fetching open issues from rust-lang/rust...
  Fetched page 1 (100 issues, 100 total so far)
  Fetched page 2 (100 issues, 200 total so far)
  ...
  Fetched page 100 (50 issues, 10,000+ total so far)

Fetched 10,000+ open issues in 100+ pages

Checking deleted files against open issues...

⚠️  Out-of-sync issues (ALL files deleted but issue still open):

  • Issue #98765: tests/crashes/98765.rs deleted in PR #147900 (commit abc12345, 2024-03-15)
    Issue: https://github.com/rust-lang/rust/issues/98765
    PR: https://github.com/rust-lang/rust/pull/147900

  • Issue #98766: 2 files deleted (98766-1.rs, 98766-2.rs)
    Issue: https://github.com/rust-lang/rust/issues/98766
    PR: https://github.com/rust-lang/rust/pull/147901

ℹ️  Partial cleanup (some files deleted, others remain):

  • Issue #99123: 1 file(s) deleted, 2 remain
    Deleted: 99123-1.rs
    Issue: https://github.com/rust-lang/rust/issues/99123

─────────────────────────────────────────────────
Statistics:
  Total crash test files deleted: 248
  Files with open issues: 4 (1.6%)
  Files with closed issues: 244 (98.4%)

  Total open issues in rust-lang/rust: 10,000+

  Issues fully cleaned up: 243
  Issues needing attention: 2
  Issues with partial cleanup: 1
─────────────────────────────────────────────────

⚠️  Found 2 issue(s) that need attention.

These issues should either:
  1. Be closed (if the issue is actually fixed)
  2. Have tests restored (if removed by mistake)
```

## How It Works

1. **Git History Scan**: Walks through commit history (optionally filtered by date)
2. **Deletion Detection**: Identifies commits that deleted files from `tests/crashes/`
3. **Issue Extraction**: Parses filenames to extract issue numbers (e.g., `12345.rs` → issue #12345)
4. **Current File Scan**: Lists all currently existing crash test files
5. **Load/Fetch Open Issues**:
   - **First run**: Fetches ALL open issues via ~100 paginated API requests, saves to `.cache/`
   - **Subsequent runs**: Loads from cache (instant, 0 API calls)
   - **Manual refresh**: Use `--refresh-cache` flag to update cache
6. **Categorization**: Groups deleted files by issue and checks if any files remain:
   - **Fully deleted**: All files for an issue are gone → check if issue is still open
   - **Partially deleted**: Some files remain → informational only (issue should be open)
7. **Report Generation**: Three-section report with actionable items and statistics

## API Rate Limits

### Without Authentication (60 requests/hour)
- May not be sufficient (~100 requests needed)
- Consider using authentication for reliable operation

### With Authentication (5,000 requests/hour)
- Recommended for frequent use
- No rate limit concerns for normal usage
- Faster execution

## Testing

To test the tool, clone the Rust repository:

```bash
cd /path/to/rust-repo-helpers
git clone https://github.com/rust-lang/rust.git
```

Then run the tool:

```bash
# Without authentication (works but slower)
cargo run -- ./rust --from 2024-01-01

# With authentication (recommended)
export GITHUB_TOKEN=ghp_your_token_here
cargo run -- ./rust --from 2024-01-01 --verbose
```

## Requirements

- Rust 1.70 or later
- Git repository with history
- Internet connection (for GitHub API calls)
- Optional: GitHub personal access token for higher rate limits
