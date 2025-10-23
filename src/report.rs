use crate::git::DeletedCrashTest;
use std::path::Path;

/// Print report of findings
pub fn print_report(
    fully_deleted_out_of_sync: &[(u64, Vec<&DeletedCrashTest>)],
    fully_deleted_synced: &[(u64, Vec<&DeletedCrashTest>)],
    partially_deleted: &[(u64, Vec<&DeletedCrashTest>, usize)],
    files_with_open_issues: usize,
    files_with_closed_issues: usize,
    total_open_issues: usize,
) {
    let total_files = files_with_open_issues + files_with_closed_issues;

    // Section 1: Out-of-sync issues (fully deleted but still open)
    if !fully_deleted_out_of_sync.is_empty() {
        println!("⚠️  Out-of-sync issues (ALL files deleted but issue still open):");
        println!();
        for (issue_number, files) in fully_deleted_out_of_sync {
            // Get unique PR numbers
            let pr_numbers: Vec<u64> = files
                .iter()
                .filter_map(|f| f.pr_number)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            // List deleted files
            let deleted_files: Vec<String> = files
                .iter()
                .map(|f| {
                    Path::new(&f.file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&f.file_path)
                        .to_string()
                })
                .collect();

            if files.len() == 1 {
                let file = files[0];
                if let Some(pr_number) = file.pr_number {
                    println!(
                        "  • Issue #{}: {} deleted in PR #{} (commit {}, {})",
                        issue_number,
                        file.file_path,
                        pr_number,
                        &file.commit_sha[..8],
                        file.commit_date
                    );
                } else {
                    println!(
                        "  • Issue #{}: {} deleted in commit {} ({})",
                        issue_number,
                        file.file_path,
                        &file.commit_sha[..8],
                        file.commit_date
                    );
                }
            } else {
                println!(
                    "  • Issue #{}: {} files deleted ({})",
                    issue_number,
                    files.len(),
                    deleted_files.join(", ")
                );
            }

            println!(
                "    Issue: https://github.com/rust-lang/rust/issues/{}",
                issue_number
            );
            if !pr_numbers.is_empty() {
                for pr_number in pr_numbers {
                    println!(
                        "    PR: https://github.com/rust-lang/rust/pull/{}",
                        pr_number
                    );
                }
            }
            println!();
        }
    }

    // Section 2: Partially deleted issues
    if !partially_deleted.is_empty() {
        println!("ℹ️  Partial cleanup (some files deleted, others remain):");
        println!();
        for (issue_number, files, remaining_count) in partially_deleted {
            let deleted_files: Vec<String> = files
                .iter()
                .map(|f| {
                    Path::new(&f.file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&f.file_path)
                        .to_string()
                })
                .collect();

            println!(
                "  • Issue #{}: {} file(s) deleted, {} remain",
                issue_number,
                files.len(),
                remaining_count
            );
            println!("    Deleted: {}", deleted_files.join(", "));
            println!(
                "    Issue: https://github.com/rust-lang/rust/issues/{}",
                issue_number
            );
            println!();
        }
    }

    // Section 3: Statistics
    println!("─────────────────────────────────────────────────");
    println!("Statistics:");
    println!("  Total crash test files deleted: {}", total_files);
    println!(
        "  Files with open issues: {} ({:.1}%)",
        files_with_open_issues,
        percentage(files_with_open_issues, total_files)
    );
    println!(
        "  Files with closed issues: {} ({:.1}%)",
        files_with_closed_issues,
        percentage(files_with_closed_issues, total_files)
    );
    println!();
    println!("  Total open issues in rust-lang/rust: {}", total_open_issues);
    println!();
    println!(
        "  Issues fully cleaned up: {}",
        fully_deleted_synced.len()
    );
    println!(
        "  Issues needing attention: {}",
        fully_deleted_out_of_sync.len()
    );
    if !partially_deleted.is_empty() {
        println!(
            "  Issues with partial cleanup: {}",
            partially_deleted.len()
        );
    }
    println!("─────────────────────────────────────────────────");

    // Final message
    if fully_deleted_out_of_sync.is_empty() {
        println!("\n✅ All fully deleted crash tests have properly closed issues!");
    } else {
        println!(
            "\n⚠️  Found {} issue(s) that need attention.",
            fully_deleted_out_of_sync.len()
        );
        println!("\nThese issues should either:");
        println!("  1. Be closed (if the issue is actually fixed)");
        println!("  2. Have tests restored (if removed by mistake)");
    }
}

fn percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}
