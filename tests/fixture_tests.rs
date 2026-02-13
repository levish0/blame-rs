use blame_rs::{BlameError, BlameOptions, BlameRevision, DiffAlgorithm, blame, blame_with_options};
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

    let mut contents = Vec::new();
    let mut rev_idx = 0;

    loop {
        let rev_file = fixture_path.join(format!("rev{}.txt", rev_idx));
        if !rev_file.exists() {
            break;
        }

        let content = fs::read_to_string(&rev_file)
            .unwrap_or_else(|_| panic!("Failed to read {:?}", rev_file));
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

    println!("\nRevisions:");
    for (idx, content) in contents.iter().enumerate() {
        println!("  Rev {}: {:?}", idx, content.trim());
    }

    let expected_file = fixture_path.join("expected.json");
    let expected_str = fs::read_to_string(&expected_file)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", expected_file));
    let expected: Vec<ExpectedLine> =
        serde_json::from_str(&expected_str).expect("Failed to parse expected.json");

    let options = BlameOptions { algorithm };
    let result = blame_with_options(&revisions, options).expect("Blame failed");

    println!("\nBlame Results:");
    println!("{:<6} {:<10} Content", "Line", "Revision");
    println!("{}", "-".repeat(60));
    for line in result.lines() {
        println!(
            "{:<6} {:<10} {}",
            line.line_number,
            format!("Rev {}", line.revision_metadata.revision),
            line.content.trim_end()
        );
    }

    assert_eq!(
        result.len(),
        expected.len(),
        "Line count mismatch in {}",
        fixture_dir
    );

    for exp in expected {
        let line = result
            .get_line(exp.line)
            .unwrap_or_else(|| panic!("Line {} not found", exp.line));

        assert_eq!(
            line.revision_metadata.revision, exp.revision,
            "Line {} in {}: expected revision {}, got {}",
            exp.line, fixture_dir, exp.revision, line.revision_metadata.revision
        );
    }

    println!("\n[ok] {} ({}) passed", fixture_dir, algo_name);
}

fn run_with_all_algorithms<F>(mut f: F)
where
    F: FnMut(DiffAlgorithm),
{
    f(DiffAlgorithm::Myers);
    f(DiffAlgorithm::Patience);
}

fn normalize_line(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
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

#[test]
fn test_empty_revisions_returns_error() {
    let revisions: Vec<BlameRevision<'static, TestMetadata>> = Vec::new();

    let err = blame(&revisions).expect_err("empty revisions should return an error");
    assert!(matches!(err, BlameError::EmptyRevisions));
}

#[test]
fn test_single_revision_preserves_first_metadata() {
    let content = "alpha\nbeta\ngamma";
    let revisions = vec![BlameRevision {
        content,
        metadata: Rc::new(TestMetadata { revision: 0 }),
    }];

    run_with_all_algorithms(|algorithm| {
        let result = blame_with_options(&revisions, BlameOptions { algorithm })
            .expect("single revision should succeed");

        assert_eq!(result.len(), 3);
        assert_eq!(
            normalize_line(result.get_line(0).expect("line 0").content),
            "alpha"
        );
        assert_eq!(
            normalize_line(result.get_line(1).expect("line 1").content),
            "beta"
        );
        assert_eq!(
            normalize_line(result.get_line(2).expect("line 2").content),
            "gamma"
        );

        for line in result.lines() {
            assert_eq!(line.revision_metadata.revision, 0);
        }
    });
}

#[test]
fn test_trailing_newline_no_extra_line() {
    let rev0 = "a\nb\n";
    let rev1 = "a\nb\nc\n";
    let revisions = vec![
        BlameRevision {
            content: rev0,
            metadata: Rc::new(TestMetadata { revision: 0 }),
        },
        BlameRevision {
            content: rev1,
            metadata: Rc::new(TestMetadata { revision: 1 }),
        },
    ];

    run_with_all_algorithms(|algorithm| {
        let result = blame_with_options(&revisions, BlameOptions { algorithm })
            .expect("trailing newline inputs should succeed");

        assert_eq!(result.len(), 3);
        assert_eq!(
            normalize_line(result.get_line(0).expect("line 0").content),
            "a"
        );
        assert_eq!(
            normalize_line(result.get_line(1).expect("line 1").content),
            "b"
        );
        assert_eq!(
            normalize_line(result.get_line(2).expect("line 2").content),
            "c"
        );
        assert_eq!(
            result
                .get_line(0)
                .expect("line 0")
                .revision_metadata
                .revision,
            0
        );
        assert_eq!(
            result
                .get_line(1)
                .expect("line 1")
                .revision_metadata
                .revision,
            0
        );
        assert_eq!(
            result
                .get_line(2)
                .expect("line 2")
                .revision_metadata
                .revision,
            1
        );
    });
}

#[test]
fn test_crlf_inputs_are_handled() {
    let rev0 = "a\r\nb\r\n";
    let rev1 = "a\r\nb\r\nc\r\n";
    let revisions = vec![
        BlameRevision {
            content: rev0,
            metadata: Rc::new(TestMetadata { revision: 0 }),
        },
        BlameRevision {
            content: rev1,
            metadata: Rc::new(TestMetadata { revision: 1 }),
        },
    ];

    run_with_all_algorithms(|algorithm| {
        let result = blame_with_options(&revisions, BlameOptions { algorithm })
            .expect("CRLF inputs should succeed");

        assert_eq!(result.len(), 3);
        assert_eq!(
            normalize_line(result.get_line(0).expect("line 0").content),
            "a"
        );
        assert_eq!(
            normalize_line(result.get_line(1).expect("line 1").content),
            "b"
        );
        assert_eq!(
            normalize_line(result.get_line(2).expect("line 2").content),
            "c"
        );
        assert_eq!(
            result
                .get_line(0)
                .expect("line 0")
                .revision_metadata
                .revision,
            0
        );
        assert_eq!(
            result
                .get_line(1)
                .expect("line 1")
                .revision_metadata
                .revision,
            0
        );
        assert_eq!(
            result
                .get_line(2)
                .expect("line 2")
                .revision_metadata
                .revision,
            1
        );
    });
}

#[test]
fn test_reordered_lines_do_not_panic() {
    let rev0 = "a\nb\nc\n";
    let rev1 = "b\na\nc\n";
    let revisions = vec![
        BlameRevision {
            content: rev0,
            metadata: Rc::new(TestMetadata { revision: 0 }),
        },
        BlameRevision {
            content: rev1,
            metadata: Rc::new(TestMetadata { revision: 1 }),
        },
    ];

    run_with_all_algorithms(|algorithm| {
        let result = blame_with_options(&revisions, BlameOptions { algorithm })
            .expect("reordered lines should not panic");

        assert_eq!(result.len(), 3);
        assert_eq!(
            normalize_line(result.get_line(0).expect("line 0").content),
            "b"
        );
        assert_eq!(
            normalize_line(result.get_line(1).expect("line 1").content),
            "a"
        );
        assert_eq!(
            normalize_line(result.get_line(2).expect("line 2").content),
            "c"
        );
    });
}
