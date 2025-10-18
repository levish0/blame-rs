use crate::types::{
    BlameError, BlameLine, BlameOptions, BlameResult, BlameRevision, DiffAlgorithm,
};
use similar::{Algorithm, TextDiff};

#[derive(Clone, Debug)]
struct LineOrigin<T> {
    content: String,
    metadata: T,
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
///
/// # Example
///
/// ```ignore
/// use blame_rs::{blame, BlameRevision};
///
/// #[derive(Clone, Debug)]
/// struct CommitInfo {
///     hash: String,
///     author: String,
/// }
///
/// let revisions = vec![
///     BlameRevision {
///         content: "line 1\nline 2",
///         metadata: CommitInfo { hash: "abc123".into(), author: "Alice".into() },
///     },
///     BlameRevision {
///         content: "line 1\nline 2\nline 3",
///         metadata: CommitInfo { hash: "def456".into(), author: "Bob".into() },
///     },
/// ];
///
/// let result = blame(&revisions)?;
/// ```
pub fn blame<T: Clone>(revisions: &[BlameRevision<T>]) -> Result<BlameResult<T>, BlameError> {
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
pub fn blame_with_options<T: Clone>(
    revisions: &[BlameRevision<T>],
    options: BlameOptions,
) -> Result<BlameResult<T>, BlameError> {
    if revisions.is_empty() {
        return Err(BlameError::EmptyRevisions);
    }

    let similar_algorithm = match options.algorithm {
        DiffAlgorithm::Myers => Algorithm::Myers,
        DiffAlgorithm::Patience => Algorithm::Patience,
    };

    let first_revision = &revisions[0];

    let init_diff = TextDiff::configure()
        .algorithm(similar_algorithm)
        .diff_lines("", first_revision.content);
    let mut line_origins: Vec<LineOrigin<T>> = Vec::new();

    for change in init_diff.iter_all_changes() {
        use similar::ChangeTag;
        if change.tag() == ChangeTag::Insert {
            line_origins.push(LineOrigin {
                content: change.value().to_string(),
                metadata: first_revision.metadata.clone(),
            });
        }
    }

    // Forward iteration: track each line's origin through revisions
    for i in 0..revisions.len() - 1 {
        let old_content = revisions[i].content;
        let new_content = revisions[i + 1].content;
        let new_metadata = &revisions[i + 1].metadata;

        let diff = TextDiff::configure()
            .algorithm(similar_algorithm)
            .diff_lines(old_content, new_content);

        let mut new_line_origins: Vec<LineOrigin<T>> = Vec::new();

        for change in diff.iter_all_changes() {
            use similar::ChangeTag;

            match change.tag() {
                ChangeTag::Equal => {
                    let old_line_num = change.old_index().unwrap();
                    if let Some(origin) = line_origins.get(old_line_num) {
                        new_line_origins.push(origin.clone());
                    }
                }
                ChangeTag::Insert => {
                    new_line_origins.push(LineOrigin {
                        content: change.value().to_string(),
                        metadata: new_metadata.clone(),
                    });
                }
                ChangeTag::Delete => {}
            }
        }

        line_origins = new_line_origins;
    }

    let blame_lines: Vec<BlameLine<T>> = line_origins
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
