# Deployment Options for rust-crash-audit

This document explores different approaches for running the crash test audit tool regularly on the rust-lang/rust repository.

## Requirement Summary
- **Frequency**: Monthly execution
- **Purpose**: Identify crash test files that were deleted but their associated GitHub issues remain open
- **Output**: List of out-of-sync issues that may need to be closed or investigated

---

## Option 1: GitHub Actions Workflow ⭐ RECOMMENDED

### Overview
Create a GitHub Actions workflow in the rust-lang/rust repository that runs on a monthly schedule.

### Pros
- ✅ **No external infrastructure needed** - runs on GitHub's infrastructure
- ✅ **Built-in authentication** - `GITHUB_TOKEN` automatically available
- ✅ **Direct repository access** - has full git history
- ✅ **Transparent** - workflow runs are visible to all contributors
- ✅ **Free for public repos** - no cost
- ✅ **Easy to maintain** - standard YAML workflow
- ✅ **Can post results** - create issues, comments, or artifacts
- ✅ **Manual trigger option** - can run on-demand via `workflow_dispatch`

### Cons
- ⚠️ Requires adding the tool to the rust-lang/rust repository (or installing from a separate repo)
- ⚠️ Results need to be posted somewhere (issue, artifact, or external service)
- ⚠️ Limited to 6 hours max execution time (but our tool runs in ~2-3 minutes)

### Implementation Approaches

#### 1a. Add tool to rust-lang/rust repository
```yaml
# .github/workflows/crash-audit.yml
name: Crash Test Audit

on:
  schedule:
    - cron: '0 0 1 * *'  # Monthly on the 1st at midnight UTC
  workflow_dispatch:      # Allow manual runs

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need full git history

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Build audit tool
        run: |
          cd tools/crash-audit
          cargo build --release

      - name: Run audit (last 6 months)
        id: audit
        run: |
          FROM_DATE=$(date -d '6 months ago' '+%Y-%m-%d')
          ./tools/crash-audit/target/release/rust-crash-audit . --from $FROM_DATE > audit-results.md
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        continue-on-error: true

      - name: Create issue with results
        if: steps.audit.outcome == 'success'
        uses: peter-evans/create-issue-from-file@v5
        with:
          title: 'Monthly Crash Test Audit Results - ${{ github.run_id }}'
          content-filepath: audit-results.md
          labels: T-compiler, A-testsuite
```

#### 1b. Install from external repository
```yaml
# .github/workflows/crash-audit.yml
name: Crash Test Audit

on:
  schedule:
    - cron: '0 0 1 * *'
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout rust-lang/rust
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install rust-crash-audit
        run: cargo install --git https://github.com/your-org/rust-repo-helpers rust-crash-audit

      - name: Run audit
        run: rust-crash-audit . --from 2024-01-01
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      # Post results...
```

### Output Options
1. **Create GitHub Issue**: Post results as a monthly tracking issue
2. **Update tracking issue**: Update a single issue with latest results
3. **Workflow artifact**: Save results file for download
4. **Post to Zulip**: Use webhook to notify team chat
5. **Fail workflow**: Exit with error if issues found (blocking approach)

### Estimated Setup Time
- 1-2 hours to create and test workflow
- Need approval from rust-lang/rust maintainers

---

## Option 2: Integrate with triagebot

### Overview
The Rust project already uses [triagebot](https://github.com/rust-lang/triagebot) for PR automation. Could add crash audit as a triagebot feature.

### Pros
- ✅ **Consistent with existing tooling** - Rust team already maintains triagebot
- ✅ **Could trigger on events** - run when crash tests are modified in PRs
- ✅ **Existing notification channels** - triagebot can post to Zulip, GitHub, etc.
- ✅ **Shared infrastructure** - no new services to maintain

### Cons
- ⚠️ **Different language** - triagebot is written in Rust, but this requires integration work
- ⚠️ **Requires changes to triagebot** - need buy-in from triagebot maintainers
- ⚠️ **More complex** - involves modifying existing production bot
- ⚠️ **Longer development time** - need to learn triagebot architecture
- ⚠️ **Coupling** - ties audit functionality to triagebot's lifecycle

### Implementation Approach
1. Add `rust-crash-audit` as a dependency to triagebot
2. Add monthly scheduled job in triagebot
3. Integrate with triagebot's notification system
4. Alternatively: Add PR comment feature when `tests/crashes/` is modified

### Estimated Setup Time
- 1-2 weeks (need to coordinate with triagebot team, review process, etc.)

---

## Option 3: Standalone Service/Bot

### Overview
Deploy as an independent service (AWS Lambda, Heroku, fly.io, etc.) that runs on a schedule.

### Pros
- ✅ **Independent** - not tied to any specific infrastructure
- ✅ **Full control** - can customize scheduling, notifications, etc.
- ✅ **Reusable** - could monitor multiple repositories

### Cons
- ⚠️ **Requires hosting** - need to pay for and maintain infrastructure
- ⚠️ **Authentication management** - need to securely store GitHub token
- ⚠️ **More complex** - web server, error handling, monitoring
- ⚠️ **Overkill** - too much infrastructure for a simple monthly task
- ⚠️ **Cost** - even minimal hosting costs money

### Implementation Options
- **AWS Lambda + EventBridge**: Serverless function triggered monthly
- **GitHub App**: Full-fledged GitHub integration
- **Heroku Scheduler**: Simple scheduled jobs
- **fly.io**: Lightweight hosting

### Estimated Setup Time
- 3-5 days (infrastructure, deployment, monitoring, testing)

---

## Option 4: Local Cron Job

### Overview
Set up a cron job on a maintainer's local machine or server.

### Pros
- ✅ **Simple setup** - just add to crontab
- ✅ **No external dependencies** - runs locally
- ✅ **Full control** - can customize easily

### Cons
- ⚠️ **Single point of failure** - if machine is off, job doesn't run
- ⚠️ **Not transparent** - team can't see when it runs
- ⚠️ **Manual maintenance** - someone needs to maintain it
- ⚠️ **Not scalable** - doesn't work well for team collaboration
- ⚠️ **Results distribution** - need to manually share results

### Implementation
```bash
# Crontab entry (runs on 1st of each month at 2am)
0 2 1 * * cd /path/to/rust && /path/to/rust-crash-audit . --from 2024-01-01 | mail -s "Crash Audit Results" team@example.com
```

### Estimated Setup Time
- 30 minutes to 1 hour

---

## Option 5: Manual Execution Only

### Overview
Don't automate - just run the tool manually when needed.

### Pros
- ✅ **Zero setup** - tool is already built
- ✅ **No infrastructure** - nothing to maintain
- ✅ **Flexible** - run whenever needed

### Cons
- ⚠️ **Easy to forget** - no automatic reminders
- ⚠️ **Inconsistent** - depends on someone remembering
- ⚠️ **Not proactive** - might miss issues for months
- ⚠️ **Manual effort** - someone has to remember to run it

### Implementation
Just document how to run it:
```bash
./rust-crash-audit /path/to/rust --from 2024-01-01
```

### Estimated Setup Time
- 0 (already done)

---

## Comparison Matrix

| Option | Complexity | Setup Time | Maintenance | Cost | Transparency | Reliability |
|--------|-----------|------------|-------------|------|--------------|-------------|
| **GitHub Actions** | Low | 1-2 hours | Low | Free | High | High |
| **triagebot** | High | 1-2 weeks | Medium | Free | High | High |
| **Standalone Service** | High | 3-5 days | High | $5-50/mo | Medium | High |
| **Local Cron** | Low | 30 min | Medium | Free | Low | Medium |
| **Manual Only** | None | 0 | None | Free | Low | Low |

---

## Recommendation: GitHub Actions (Option 1b)

**Why GitHub Actions?**
1. **Best balance** of simplicity, reliability, and maintainability
2. **Free and transparent** - no cost, everyone can see it running
3. **Quick to set up** - 1-2 hours vs weeks for other options
4. **Standard approach** - rust-lang/rust already uses many GitHub Actions
5. **Easy to modify** - team can update the workflow as needed

**Recommended Implementation:**
- Use Option 1b (install from external repo) to keep tool separate
- Create monthly issue with results
- Allow manual runs via `workflow_dispatch`
- Run on the 1st of each month
- Scan last 6 months of history

**Next Steps:**
1. Publish `rust-crash-audit` to a public repository (or keep in rust-repo-helpers)
2. Create GitHub Actions workflow file
3. Test workflow in a fork
4. Submit PR to rust-lang/rust with the workflow
5. Coordinate with rust-lang/rust team for approval

---

## Alternative: Start with Manual, Move to GitHub Actions

If you're unsure about the automation approach:
1. **Phase 1**: Document manual execution process, run it monthly yourself
2. **Phase 2**: After 2-3 months, evaluate if automation is needed
3. **Phase 3**: Implement GitHub Actions based on learnings

This validates the tool's usefulness before investing in automation infrastructure.
