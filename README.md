<div align="center">

# blame-rs

Line-by-line authorship tracking for revisioned text using proven diff algorithms.

[![Crates.io](https://img.shields.io/crates/v/blame-rs.svg)](https://crates.io/crates/blame-rs)
[![Documentation](https://docs.rs/blame-rs/badge.svg)](https://docs.rs/blame-rs)
[![Downloads](https://img.shields.io/crates/d/blame-rs.svg)](https://crates.io/crates/blame-rs)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.88.0+-orange.svg?logo=rust)](https://www.rust-lang.org/)

</div>

---

## 🌟 Overview

`blame-rs` is a Rust library for **line-by-line authorship tracking** in revisioned text.
Track which revision introduced each line in your documents with a flexible in-memory API.

---

## ✨ Features

- **Generic metadata API**: Attach any metadata type to revisions (commit hashes, authors, timestamps, etc.)
- **Multiple diff algorithms**: Support for Myers (default) and Patience algorithms via the `similar` crate
- **Forward tracking**: Efficiently traces line origins from oldest to newest revision
- **High performance**:
  - Zero-copy line tracking with `&str` references (no string allocations)
  - Shared metadata via `Rc<T>` (single clone per revision instead of per line)
  - Pre-allocated vectors (minimal heap reallocations)
- **Well tested**: Comprehensive test suite with fixture-based scenarios

### Supported Diff Algorithms

#### Myers (Default)
- **Standard diff algorithm**: Fast and reliable for most use cases
- **O(ND) complexity**: Efficient for typical text changes
- **Widely used**: Same algorithm used in many diff tools

#### Patience
- **High-quality diffs**: Produces more intuitive results for code
- **Unique line matching**: Better handling of code movement
- **Ideal for**: Source code with unique lines and structural changes

---

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
blame-rs = "0.1.0"
```

### Basic Usage

```rust
use blame_rs::{blame, BlameRevision};
use std::rc::Rc;

#[derive(Debug)]
struct CommitInfo {
    hash: String,
    author: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let revisions = vec![
        BlameRevision {
            content: "line 1\nline 2",
            metadata: Rc::new(CommitInfo {
                hash: "abc123".to_string(),
                author: "Alice".to_string(),
            }),
        },
        BlameRevision {
            content: "line 1\nline 2\nline 3",
            metadata: Rc::new(CommitInfo {
                hash: "def456".to_string(),
                author: "Bob".to_string(),
            }),
        },
    ];

    let result = blame(&revisions)?;

    for line in result.lines() {
        println!(
            "Line {}: {} (from {})",
            line.line_number,
            line.content.trim(),
            line.revision_metadata.author
        );
    }

    Ok(())
}
```

**Output:**
```
Line 0: line 1 (from Alice)
Line 1: line 2 (from Alice)
Line 2: line 3 (from Bob)
```

### Advanced Configuration

```rust
use blame_rs::{blame_with_options, BlameOptions, BlameRevision, DiffAlgorithm};

let options = BlameOptions {
    algorithm: DiffAlgorithm::Patience,
};

let result = blame_with_options(&revisions, options)?;

for line in result.lines() {
    println!(
        "{:<6} {:<10} {:<15} {}",
        line.line_number + 1,
        &line.revision_metadata.hash[..6],
        line.revision_metadata.author,
        line.content.trim_end()
    );
}
```

---

## 🔍 How It Works

The algorithm works by:

1. **Initialize**: Starting with the first (oldest) revision, assigning all lines to that revision
2. **Iterate forward**: Processing each consecutive revision pair
3. **Compute diff**: Using the selected diff algorithm (Myers or Patience)
4. **Track origins**: For each line in the newer revision:
   - **Equal** → Keep original metadata (unchanged line)
   - **Insert** → Assign current revision metadata (new line)
   - **Delete** → Remove from tracking (deleted line)

This **forward-tracking approach** ensures accurate line attribution even when lines are moved, modified, or deleted across multiple revisions.

### Example Workflow

```
Rev 0 (Alice):          Rev 1 (Bob):           Rev 2 (Charlie):
fn main() {             fn main() {            fn main() {
  println!("Hello");      println!("Hello");     println!("Rust!");  ← Changed
}                         println!("World");     println!("World");
                        }                      }

Result:
Line 0: fn main() { → Alice (unchanged since Rev 0)
Line 1: println!("Rust!"); → Charlie (changed in Rev 2)
Line 2: println!("World"); → Bob (added in Rev 1)
Line 3: } → Alice (unchanged since Rev 0)
```

---

## 📦 Examples

See the `examples/` directory for detailed usage:

- **`basic_usage.rs`**: Demonstrates blame with multiple revisions and formatted table output
- **`debug_example.rs`**: Shows detailed blame output with revision content for debugging

Run examples with:

```bash
cargo run --example basic_usage
cargo run --example debug_example
```

---

## 🧪 Testing & Quality

### Comprehensive Test Suite

The library includes extensive testing with:
- **Fixture-based tests**: Multiple real-world scenarios in `tests/fixtures/`
- **Both algorithms tested**: Every fixture runs with Myers and Patience
- **Test scenarios include**:
  - Simple line additions
  - Multiple revisions with incremental changes
  - Line modifications and deletions
  - Complex multi-revision histories

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test -- --nocapture

# Run specific fixture test
cargo test test_multiple_revisions_myers -- --nocapture --exact
```

### Test Output Example

```
================================================================================
Testing: multiple_revisions (Algorithm: Myers)
================================================================================

Revisions:
  Rev 0: "a\nb"
  Rev 1: "a\nb\nc"
  Rev 2: "a\nb\nc\nd"

Blame Results:
Line   Revision   Content
------------------------------------------------------------
0      Rev 0      a
1      Rev 0      b
2      Rev 1      c
3      Rev 2      d

✓ multiple_revisions (Myers) passed
```

---

## 📚 API Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

Key types:
- `BlameRevision<'a, T>`: Represents a revision with content (`&'a str`) and metadata (`Rc<T>`)
  - `content: &'a str` - Zero-copy reference to revision content
  - `metadata: Rc<T>` - Shared reference-counted metadata (no `T: Clone` required)
- `BlameLine<'a, T>`: A single line with its origin information
  - `content: &'a str` - Zero-copy reference to the original line
  - `revision_metadata: Rc<T>` - Shared reference to revision metadata
- `BlameResult<'a, T>`: Collection of blamed lines
- `BlameOptions`: Configuration for the blame operation
- `DiffAlgorithm`: Myers or Patience algorithm selection

**Note**: The library uses zero-copy string slices (`&str`) and shared metadata (`Rc<T>`) for optimal performance. Metadata types don't need to implement `Clone`.

---

## 🏗️ Requirements

- **Rust**: 1.88.0 or later (uses 2024 edition)
- **Dependencies**:
  - `similar` (2.7.0) – Text diffing algorithms
  - `thiserror` (2.0.17) – Error handling

---

## 🙏 Acknowledgments

### Core Technologies
- [**Rust**](https://www.rust-lang.org/) – Systems programming language with safety and performance
- [**similar**](https://github.com/mitsuhiko/similar) – Powerful text diffing library by Armin Ronacher

### Special Thanks
- **Open Source Community**: For the incredible tools and libraries
- **Contributors**: Everyone who improves `blame-rs`
- **similar maintainers**: For providing excellent diff algorithms in Rust