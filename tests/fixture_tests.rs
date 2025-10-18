use blame_rs::{BlameOptions, BlameRevision, DiffAlgorithm, blame_with_options};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
struct TestMetadata {
    revision: usize,
}

#[derive(Deserialize)]
struct ExpectedLine {
    line: usize,
    revision: usize,
}

fn run_fixture_test(fixture_dir: &str, algorithm: DiffAlgorithm) {
    let algo_name = match algorithm {
        DiffAlgorithm::Myers => "Myers",
        DiffAlgorithm::Patience => "Patience",
    };

    println!("\n{}", "=".repeat(80));
    println!("Testing: {} (Algorithm: {})", fixture_dir, algo_name);
    println!("{}", "=".repeat(80));

    let fixture_path = Path::new("tests/fixtures").join(fixture_dir);

    // Read all revision files
    let mut contents = Vec::new();
    let mut rev_idx = 0;

    loop {
        let rev_file = fixture_path.join(format!("rev{}.txt", rev_idx));
        if !rev_file.exists() {
            break;
        }

        let content =
            fs::read_to_string(&rev_file).expect(&format!("Failed to read {:?}", rev_file));
        contents.push(content);

        rev_idx += 1;
    }

    let revisions: Vec<BlameRevision<TestMetadata>> = contents
        .iter()
        .enumerate()
        .map(|(idx, content)| BlameRevision {
            content: content.as_str(),
            metadata: Rc::new(TestMetadata { revision: idx }),
        })
        .collect();

    assert!(
        !revisions.is_empty(),
        "No revision files found in {}",
        fixture_dir
    );

    // Print revision contents
    println!("\nRevisions:");
    for (idx, content) in contents.iter().enumerate() {
        println!("  Rev {}: {:?}", idx, content.trim());
    }

    // Read expected results
    let expected_file = fixture_path.join("expected.json");
    let expected_str =
        fs::read_to_string(&expected_file).expect(&format!("Failed to read {:?}", expected_file));
    let expected: Vec<ExpectedLine> =
        serde_json::from_str(&expected_str).expect("Failed to parse expected.json");

    // Run blame
    let options = BlameOptions { algorithm };
    let result = blame_with_options(&revisions, options).expect("Blame failed");

    // Print blame results
    println!("\nBlame Results:");
    println!("{:<6} {:<10} {}", "Line", "Revision", "Content");
    println!("{}", "-".repeat(60));
    for line in result.lines() {
        println!(
            "{:<6} {:<10} {}",
            line.line_number,
            format!("Rev {}", line.revision_metadata.revision),
            line.content.trim_end()
        );
    }

    // Verify results
    assert_eq!(
        result.len(),
        expected.len(),
        "Line count mismatch in {}",
        fixture_dir
    );

    for exp in expected {
        let line = result
            .get_line(exp.line)
            .expect(&format!("Line {} not found", exp.line));

        assert_eq!(
            line.revision_metadata.revision, exp.revision,
            "Line {} in {}: expected revision {}, got {}",
            exp.line, fixture_dir, exp.revision, line.revision_metadata.revision
        );
    }

    println!("\n✓ {} ({}) passed", fixture_dir, algo_name);
}

#[test]
fn test_simple_add_myers() {
    run_fixture_test("simple_add", DiffAlgorithm::Myers);
}

#[test]
fn test_simple_add_patience() {
    run_fixture_test("simple_add", DiffAlgorithm::Patience);
}

#[test]
fn test_multiple_revisions_myers() {
    run_fixture_test("multiple_revisions", DiffAlgorithm::Myers);
}

#[test]
fn test_multiple_revisions_patience() {
    run_fixture_test("multiple_revisions", DiffAlgorithm::Patience);
}

#[test]
fn test_line_modification_myers() {
    run_fixture_test("line_modification", DiffAlgorithm::Myers);
}

#[test]
fn test_line_modification_patience() {
    run_fixture_test("line_modification", DiffAlgorithm::Patience);
}
