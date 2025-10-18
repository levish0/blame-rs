//! # blame-rs
//!
//! A Rust library for line-by-line authorship tracking in revisioned text content.
//!
//! This crate provides a Git-style blame/annotate algorithm that determines which revision
//! introduced each line in a document by analyzing a sequence of revisions.
//!
//! ## Features
//!
//! - **Generic metadata**: Attach any metadata type to revisions (commit hashes, authors, timestamps, etc.)
//! - **Multiple diff algorithms**: Support for Myers and Patience algorithms via the `similar` crate
//! - **Forward tracking**: Efficiently traces line origins from oldest to newest revision
//!
//! ## Example
//!
//! ```rust
//! use blame_rs::{blame, BlameRevision};
//!
//! #[derive(Clone, Debug)]
//! struct CommitInfo {
//!     hash: String,
//!     author: String,
//! }
//!
//! let revisions = vec![
//!     BlameRevision {
//!         content: "line 1\nline 2",
//!         metadata: CommitInfo {
//!             hash: "abc123".to_string(),
//!             author: "Alice".to_string(),
//!         },
//!     },
//!     BlameRevision {
//!         content: "line 1\nline 2\nline 3",
//!         metadata: CommitInfo {
//!             hash: "def456".to_string(),
//!             author: "Bob".to_string(),
//!         },
//!     },
//! ];
//!
//! let result = blame(&revisions).unwrap();
//! for line in result.lines() {
//!     println!("{}: {}", line.revision_metadata.author, line.content);
//! }
//! ```

mod blame;
mod types;

pub use blame::{blame, blame_with_options};
pub use types::{BlameError, BlameLine, BlameOptions, BlameResult, BlameRevision, DiffAlgorithm};
