use blame_rs::DiffAlgorithm::Patience;
use blame_rs::{BlameOptions, BlameRevision, blame_with_options};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

#[derive(Debug, Clone)]
struct CommitInfo {
    hash: String,
    author: String,
    message: String,
    timestamp: String,
}

// Generate a git-like hash from commit metadata
fn generate_commit_hash(author: &str, message: &str, timestamp: &str) -> String {
    let mut hasher = DefaultHasher::new();
    author.hash(&mut hasher);
    message.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    let hash_value = hasher.finish();
    format!("{:016x}", hash_value)[..7].to_string()
}

fn parse_revision_index(path: &Path) -> usize {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix("rev"))
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or(usize::MAX)
}

fn main() {
    // Read all revision files from the revisions directory
    let revisions_dir = "examples/revisions";
    let mut revision_files: Vec<_> = fs::read_dir(revisions_dir)
        .expect("Failed to read revisions directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "txt" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort by numeric revision index (rev2.txt before rev10.txt)
    revision_files.sort_by_key(|path| parse_revision_index(path));

    let num_revisions = revision_files.len();
    println!("Found {} revision files", num_revisions);
    println!();

    // Read file contents
    let contents: Vec<String> = revision_files
        .iter()
        .map(|path| {
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()))
        })
        .collect();

    // Generate metadata dynamically based on number of files found
    let authors = vec![
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
        "Kelly", "Liam", "Maria", "Nathan", "Olivia", "Peter", "Quinn", "Rachel", "Steve", "Tina",
    ];

    let message_templates = vec![
        "Initial configuration",
        "Update settings",
        "Add new features",
        "Refactor and improve",
        "Fix bugs and optimize",
        "Implement enhancements",
        "Add security improvements",
        "Performance optimization",
        "Update documentation",
        "Add error handling",
        "Improve logging",
        "Refactor codebase",
        "Add new functionality",
        "Update dependencies",
        "Improve reliability",
        "Add monitoring",
        "Enhance features",
        "Fix critical issues",
        "Improve performance",
        "Final adjustments",
    ];

    let commit_metadata: Vec<CommitInfo> = (0..num_revisions)
        .map(|i| {
            let author = authors[i % authors.len()];
            let message = message_templates[i % message_templates.len()];
            let timestamp = format!(
                "2024-{:02}-{:02} {:02}:{:02}:00",
                1 + (i / 4),        // month
                1 + ((i * 5) % 28), // day
                9 + (i % 8),        // hour
                (i * 15) % 60       // minute
            );
            let hash = generate_commit_hash(author, message, &timestamp);

            CommitInfo {
                hash,
                author: author.to_string(),
                message: message.to_string(),
                timestamp,
            }
        })
        .collect();

    // Create BlameRevision objects
    let revisions: Vec<BlameRevision<CommitInfo>> = contents
        .iter()
        .enumerate()
        .map(|(i, content)| BlameRevision {
            content,
            metadata: Rc::new(commit_metadata[i].clone()),
        })
        .collect();

    // Run blame with timing
    let start = Instant::now();
    let result = blame_with_options(
        &revisions,
        BlameOptions {
            algorithm: Patience,
        },
    )
    .expect("Blame operation failed");
    let duration = start.elapsed();

    // Print results
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            BLAME ANALYSIS RESULTS                             ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "{:<6} {:<10} {:<12} {:<20} Content",
        "Line", "Commit", "Author", "Timestamp"
    );
    println!("{}", "─".repeat(100));

    for line in result.lines() {
        let commit_short = &line.revision_metadata.hash;
        let content = line.content.trim_end();

        println!(
            "{:<6} {:<10} {:<12} {:<20} {}",
            line.line_number + 1,
            commit_short,
            line.revision_metadata.author,
            &line.revision_metadata.timestamp[..10], // Show only date
            content
        );
    }

    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              REVISION HISTORY                                 ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    for (i, metadata) in commit_metadata.iter().enumerate() {
        println!(
            "Rev {}: {} - {} - {} - \"{}\"",
            i, metadata.hash, metadata.author, metadata.timestamp, metadata.message
        );
    }

    // Calculate statistics
    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              CONTRIBUTION STATS                               ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let mut author_lines: HashMap<String, usize> = HashMap::new();
    for line in result.lines() {
        *author_lines
            .entry(line.revision_metadata.author.clone())
            .or_insert(0) += 1;
    }

    let total_lines = result.lines().len();
    let mut author_stats: Vec<_> = author_lines.iter().collect();
    author_stats.sort_by(|a, b| b.1.cmp(a.1));

    println!("{:<15} {:<10} {:<10}", "Author", "Lines", "Percentage");
    println!("{}", "─".repeat(40));

    for (author, count) in author_stats {
        let percentage = (*count as f64 / total_lines as f64) * 100.0;
        println!("{:<15} {:<10} {:.1}%", author, count, percentage);
    }

    println!();
    println!("Total lines: {}", total_lines);
    println!("Total revisions: {}", revisions.len());
    println!("⏱️  Blame operation took: {:?}", duration);
}
