use crate::types::{
    BlameError, BlameLine, BlameOptions, BlameResult, BlameRevision, DiffAlgorithm,
};
use similar::{Algorithm, ChangeTag, capture_diff_slices};
use std::rc::Rc;

#[derive(Debug)]
struct LineOrigin<'a, T> {
    content: &'a str,
    metadata: Rc<T>,
}

impl<'a, T> Clone for LineOrigin<'a, T> {
    fn clone(&self) -> Self {
        Self {
            content: self.content,
            metadata: Rc::clone(&self.metadata),
        }
    }
}

fn iter_lines_preserve_terminator(content: &str) -> impl Iterator<Item = &str> {
    content.split_inclusive('\n')
}

/// Performs a blame operation on a sequence of revisions to determine the origin of each line.
///
/// This function takes a slice of `BlameRevision` objects ordered chronologically (oldest to newest)
/// and computes which revision each line in the final version originated from.
///
/// # Arguments
///
/// * `revisions` - A slice of revisions ordered chronologically (oldest first, newest last)
///
/// # Returns
///
/// Returns a `BlameResult` containing each line of the final revision along with metadata
/// about which revision introduced that line.
///
/// # Errors
///
/// Returns `BlameError::EmptyRevisions` if the revisions slice is empty.
/// Returns `BlameError::InvalidInput` if diff invariants are violated.
///
/// # Example
///
/// ```ignore
/// use blame_rs::{blame, BlameRevision};
/// use std::rc::Rc;
///
/// #[derive(Debug)]
/// struct CommitInfo {
///     hash: String,
///     author: String,
/// }
///
/// let revisions = vec![
///     BlameRevision {
///         content: "line 1\nline 2",
///         metadata: Rc::new(CommitInfo { hash: "abc123".into(), author: "Alice".into() }),
///     },
///     BlameRevision {
///         content: "line 1\nline 2\nline 3",
///         metadata: Rc::new(CommitInfo { hash: "def456".into(), author: "Bob".into() }),
///     },
/// ];
///
/// let result = blame(&revisions)?;
/// ```
pub fn blame<'a, T>(
    revisions: &'a [BlameRevision<'a, T>],
) -> Result<BlameResult<'a, T>, BlameError> {
    blame_with_options(revisions, BlameOptions::default())
}

/// Performs a blame operation with custom options.
///
/// # Arguments
///
/// * `revisions` - A slice of revisions ordered chronologically (oldest first, newest last)
/// * `options` - Configuration options for the blame operation
///
/// # Returns
///
/// Returns a `BlameResult` containing each line of the final revision along with metadata
/// about which revision introduced that line.
///
/// # Errors
///
/// Returns `BlameError::EmptyRevisions` if the revisions slice is empty.
/// Returns `BlameError::InvalidInput` if diff invariants are violated.
///
/// # Example
///
/// ```ignore
/// use blame_rs::{blame_with_options, BlameOptions, BlameRevision, DiffAlgorithm};
///
/// let options = BlameOptions {
///     algorithm: DiffAlgorithm::Patience,
/// };
///
/// let result = blame_with_options(&revisions, options)?;
/// ```
pub fn blame_with_options<'a, T>(
    revisions: &'a [BlameRevision<'a, T>],
    options: BlameOptions,
) -> Result<BlameResult<'a, T>, BlameError> {
    if revisions.is_empty() {
        return Err(BlameError::EmptyRevisions);
    }

    let similar_algorithm = match options.algorithm {
        DiffAlgorithm::Myers => Algorithm::Myers,
        DiffAlgorithm::Patience => Algorithm::Patience,
    };

    let revision_lines: Vec<Vec<&'a str>> = revisions
        .iter()
        .map(|revision| iter_lines_preserve_terminator(revision.content).collect())
        .collect();

    let mut line_origins: Vec<LineOrigin<'a, T>> = Vec::with_capacity(revision_lines[0].len());
    let first_metadata = Rc::clone(&revisions[0].metadata);

    for &line in &revision_lines[0] {
        line_origins.push(LineOrigin {
            content: line,
            metadata: Rc::clone(&first_metadata),
        });
    }

    // Forward iteration: track each line's origin through revisions
    for i in 0..revisions.len() - 1 {
        let old_lines = &revision_lines[i];
        let new_lines = &revision_lines[i + 1];

        // Create shared reference to this revision's metadata
        let shared_metadata = Rc::clone(&revisions[i + 1].metadata);

        let diff_ops = capture_diff_slices(similar_algorithm, &old_lines, &new_lines);

        let mut new_line_origins: Vec<LineOrigin<'a, T>> = Vec::with_capacity(new_lines.len());

        for op in &diff_ops {
            for change in op.iter_changes(old_lines, new_lines) {
                match change.tag() {
                    ChangeTag::Equal => {
                        let old_line_num = change.old_index().ok_or_else(|| {
                            BlameError::InvalidInput(format!(
                                "diff invariant violated: Equal change had no old index at revision {}",
                                i + 1
                            ))
                        })?;
                        let origin = line_origins.get(old_line_num).ok_or_else(|| {
                            BlameError::InvalidInput(format!(
                                "diff invariant violated: old index {} out of bounds (len {}) at revision {}",
                                old_line_num,
                                line_origins.len(),
                                i + 1
                            ))
                        })?;
                        new_line_origins.push(origin.clone());
                    }
                    ChangeTag::Insert => {
                        new_line_origins.push(LineOrigin {
                            content: change.value(),
                            metadata: Rc::clone(&shared_metadata),
                        });
                    }
                    ChangeTag::Delete => {}
                }
            }
        }

        line_origins = new_line_origins;
    }

    let blame_lines: Vec<BlameLine<'a, T>> = line_origins
        .into_iter()
        .enumerate()
        .map(|(idx, origin)| BlameLine {
            line_number: idx,
            content: origin.content,
            revision_metadata: origin.metadata,
        })
        .collect();

    Ok(BlameResult::new(blame_lines))
}
