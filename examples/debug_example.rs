use blame_rs::{BlameRevision, blame};
use std::fs;

#[derive(Clone, Debug)]
struct CommitInfo {
    hash: String,
    author: String,
}

fn main() {
    let rev0 = fs::read_to_string("examples/rev0.txt").unwrap();
    let rev1 = fs::read_to_string("examples/rev1.txt").unwrap();
    let rev2 = fs::read_to_string("examples/rev2.txt").unwrap();

    println!("=== Rev 0 (Alice) ===");
    println!("{:?}", rev0);

    println!("\n=== Rev 1 (Bob) ===");
    println!("{:?}", rev1);

    println!("\n=== Rev 2 (Charlie) ===");
    println!("{:?}", rev2);

    let revisions = vec![
        BlameRevision {
            content: &rev0,
            metadata: CommitInfo {
                hash: "abc123".to_string(),
                author: "Alice".to_string(),
            },
        },
        BlameRevision {
            content: &rev1,
            metadata: CommitInfo {
                hash: "def456".to_string(),
                author: "Bob".to_string(),
            },
        },
        BlameRevision {
            content: &rev2,
            metadata: CommitInfo {
                hash: "789abc".to_string(),
                author: "Charlie".to_string(),
            },
        },
    ];

    let result = blame(&revisions).unwrap();

    println!("\n=== Blame Result ===");
    for line in result.lines() {
        println!(
            "Line {}: {:?} from {} ({})",
            line.line_number,
            line.content.trim_end(),
            line.revision_metadata.author,
            &line.revision_metadata.hash[..6]
        );
    }
}
