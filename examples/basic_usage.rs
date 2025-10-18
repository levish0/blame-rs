use blame_rs::DiffAlgorithm::Patience;
use blame_rs::{BlameOptions, BlameRevision, blame_with_options};
use std::fs;

#[derive(Clone, Debug)]
struct CommitInfo {
    hash: String,
    author: String,
    message: String,
}

fn main() {
    // Read revision files
    let rev0 = fs::read_to_string("examples/rev0.txt").expect("Failed to read rev0.txt");
    let rev1 = fs::read_to_string("examples/rev1.txt").expect("Failed to read rev1.txt");
    let rev2 = fs::read_to_string("examples/rev2.txt").expect("Failed to read rev2.txt");

    // Create revisions with metadata
    let revisions = vec![
        BlameRevision {
            content: &rev0,
            metadata: CommitInfo {
                hash: "abc123".to_string(),
                author: "Alice".to_string(),
                message: "Initial commit".to_string(),
            },
        },
        BlameRevision {
            content: &rev1,
            metadata: CommitInfo {
                hash: "def456".to_string(),
                author: "Bob".to_string(),
                message: "Add greeting message".to_string(),
            },
        },
        BlameRevision {
            content: &rev2,
            metadata: CommitInfo {
                hash: "789abc".to_string(),
                author: "Charlie".to_string(),
                message: "Update greeting and add footer".to_string(),
            },
        },
    ];

    // Run blame
    let result = blame_with_options(
        &revisions,
        BlameOptions {
            algorithm: Patience,
        },
    )
    .expect("Blame operation failed");

    // Print results
    println!("Blame Results:");
    println!("{}", "=".repeat(80));
    println!(
        "{:<6} {:<10} {:<15} {}",
        "Line", "Commit", "Author", "Content"
    );
    println!("{}", "=".repeat(80));

    for line in result.lines() {
        let commit_short = &line.revision_metadata.hash[..6];
        let content = line.content.trim_end();

        println!(
            "{:<6} {:<10} {:<15} {}",
            line.line_number + 1,
            commit_short,
            line.revision_metadata.author,
            content
        );
    }

    println!("\n{}", "=".repeat(80));
    println!("Revision Details:");
    println!("{}", "=".repeat(80));

    for (i, rev) in revisions.iter().enumerate() {
        println!(
            "Revision {}: {} - {} - \"{}\"",
            i,
            &rev.metadata.hash[..6],
            rev.metadata.author,
            rev.metadata.message
        );
    }
}
