# Rust Crash Tests Synchronization Tools

## Context

The Rust project has crash test files in `tests/crashes/...` that are linked to GitHub issues. When issues are fixed, the test files should be removed. These tools help keep tests and issues synchronized.

Reference: https://github.com/matthewjasper/rust/commit/87e5969572ebc0d6b97277d4ad06fa3f5a0b7010#diff-bb82199172373303d0db6440aee9300e8b8d5e30d39c315aa7cd91

---

## Approach 1: Triagebot Integration (PR Comments)

### Goal
Automatically comment on PRs when files in `tests/crashes/...` are modified or deleted.

### Implementation Strategy

**Technology:** Extend existing triagebot codebase

**Key Components:**
1. **Event Listener**: Hook into GitHub PR webhook events
2. **File Change Detection**: Check if PR modifies/deletes files in `tests/crashes/`
3. **Issue Extraction**: Parse filename to get issue number (e.g., `123456.rs` → #123456)
4. **Comment Generator**: Post notification comment on the PR

**Similar Existing Feature:**
- Triagebot already has functionality that comments on PRs when certain files are modified and notifies specific users
- Can use this as a reference implementation

**Configuration Options:**
```toml
[crash-tests]
enabled = true
message = "This PR modifies crash test files. Please ensure the associated issues are properly closed."
notify = ["@rust-lang/compiler"]
```

**Workflow:**
1. PR is opened/updated
2. Triagebot receives webhook event
3. Check if any files match `tests/crashes/**/*.rs`
4. If matches found:
   - Extract issue numbers from filenames
   - Post comment with list of affected tests/issues
   - Optionally verify issue status via GitHub API

**Challenges:**
- Understanding triagebot's architecture
- Webhook event handling
- Rate limiting for issue status checks
- Writing clear, actionable messages

**Estimated Effort:** 3-4 days

---

## Approach 2: Standalone Audit Tool (History Scanner)

### Goal
Scan Rust repo history to find deleted crash test files and verify their associated issues are properly closed.

### Implementation Strategy

**Technology:** Standalone Rust CLI tool

**Key Components:**
1. **Git History Traversal**: Walk through commits (optionally filtered by date)
2. **Deletion Detection**: Find commits that delete files from `tests/crashes/`
3. **Issue Extraction**: Parse filenames to get issue numbers
4. **GitHub API Client**: Check issue status (open/closed)
5. **Reporter**: Generate report of out-of-sync issues

**Command Interface:**
```bash
# Scan entire history
rust-crash-audit /path/to/rust

# Scan specific date range
rust-crash-audit /path/to/rust --from 2024-01-01 --to 2024-12-31

# Output formats
rust-crash-audit /path/to/rust --format json
rust-crash-audit /path/to/rust --format markdown

# Verbose mode with progress
rust-crash-audit /path/to/rust --verbose
```

**Output Example:**
```
Scanning Rust repository...
Date range: 2024-01-01 to 2024-12-31
Found 245 deleted crash test files

Out-of-sync issues (tests deleted but issues still open):
- Issue #98765: tests/crashes/98765.rs deleted in abc1234 (2024-03-15) - STILL OPEN
- Issue #98766: tests/crashes/98766.rs deleted in def5678 (2024-04-20) - STILL OPEN

Summary:
- Total deleted tests: 245
- Issues still open: 2 (0.8%)
- Issues properly closed: 243 (99.2%)
```

**Implementation Details:**

1. **Git Operations** (`git2-rs`):
   ```rust
   // Walk commits
   let mut revwalk = repo.revwalk()?;
   revwalk.push_head()?;

   // Filter by date if provided
   for oid in revwalk {
       let commit = repo.find_commit(oid?)?;
       // Check file deletions in diff
   }
   ```

2. **File Deletion Detection**:
   - Compare each commit with parent
   - Look for deleted files matching `tests/crashes/**/*.rs`
   - Extract issue number from filename

3. **GitHub API Integration**:
   - Use `octocrab` or `reqwest` for API calls
   - Implement rate limiting (5000 requests/hour for authenticated)
   - Cache results to avoid duplicate API calls
   - Handle authentication (GitHub token)

4. **Performance Optimizations**:
   - Use `--first-parent` for faster traversal
   - Parallel API requests (with rate limiting)
   - Optional caching layer for issue status

**Configuration:**
```toml
# .rust-crash-audit.toml
[github]
token = "ghp_..."  # or from environment
repo = "rust-lang/rust"

[scan]
# Only scan main branch
branch = "master"
# Path pattern to match
pattern = "tests/crashes/**/*.rs"

[output]
verbose = false
format = "text"
```

**Challenges:**
- Rust repo is massive (performance matters)
- GitHub API rate limiting
- Handling edge cases (renamed files, moved directories)
- Date filtering with Git
- Progress indication for long-running scans

**Estimated Effort:** 5-7 days

---

## Technology Choices

### Approach 1 (Triagebot)
- **Language**: Rust (existing codebase)
- **Dependencies**: Already in triagebot
- **Deployment**: Runs as webhook server

### Approach 2 (Standalone Tool)
- **Language**: Rust
- **Core Dependencies**:
  - `clap`: CLI argument parsing
  - `git2`: Git operations
  - `octocrab` or `reqwest`: GitHub API
  - `tokio`: Async runtime
  - `serde`: Serialization
  - `indicatif`: Progress bars
  - `chrono`: Date handling

---

## Deliverables

### Approach 1
- [ ] PR to triagebot repository
- [ ] Documentation for configuration
- [ ] Test suite for new feature
- [ ] Example configuration for rust-lang/rust

### Approach 2
- [ ] Standalone CLI tool (`rust-crash-audit`)
- [ ] README with usage instructions
- [ ] Installation guide
- [ ] Example outputs
- [ ] GitHub Actions workflow (optional)

---

## Notes

- Both tools address different aspects of the same problem
- Triagebot is proactive (catches issues in PRs)
- Audit tool is retrospective (finds existing problems)
- Together they provide comprehensive coverage
