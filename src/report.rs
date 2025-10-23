use crate::git::DeletedCrashTest;

/// Print report of findings
pub fn print_report(
    out_of_sync: &[DeletedCrashTest],
    synced: &[DeletedCrashTest],
    total_open_issues: usize,
) {
    let total = out_of_sync.len() + synced.len();

    if !out_of_sync.is_empty() {
        println!("⚠️  Out-of-sync issues (test deleted but issue still open):");
        println!();
        for file in out_of_sync {
            println!(
                "  • Issue #{}: {} deleted in {} ({})",
                file.issue_number,
                file.file_path,
                &file.commit_sha[..8],
                file.commit_date
            );
            println!(
                "    https://github.com/rust-lang/rust/issues/{}",
                file.issue_number
            );
            println!();
        }
    }

    // Summary
    println!("─────────────────────────────────────────────────");
    println!("Summary:");
    println!("  Total deleted tests: {}", total);
    println!("  Total open issues in rust-lang/rust: {}", total_open_issues);
    println!();
    println!(
        "  ⚠️  Issues still open: {} ({:.1}%)",
        out_of_sync.len(),
        percentage(out_of_sync.len(), total)
    );
    println!(
        "  ✅ Issues properly closed: {} ({:.1}%)",
        synced.len(),
        percentage(synced.len(), total)
    );
    println!("─────────────────────────────────────────────────");

    if out_of_sync.is_empty() {
        println!("\n✅ All deleted crash tests have properly closed issues!");
    } else {
        println!(
            "\n⚠️  Found {} out-of-sync issue(s) that need attention.",
            out_of_sync.len()
        );
        println!("\nThese issues should either:");
        println!("  1. Be reopened (if the crash test was removed by mistake)");
        println!("  2. Be closed (if the issue is actually fixed)");
    }
}

fn percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}
