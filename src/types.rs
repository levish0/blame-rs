use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct BlameRevision<'a, T> {
    pub content: Cow<'a, str>,
    pub metadata: T,
}

#[derive(Debug, Clone)]
pub struct BlameLine<T> {
    pub line_number: usize,
    pub content: String,
    pub revision_metadata: T,
}

/// The result of a blame operation, containing all lines with their origin information
#[derive(Debug, Clone)]
pub struct BlameResult<T> {
    lines: Vec<BlameLine<T>>,
}

impl<T> BlameResult<T> {
    /// Create a new BlameResult from a vector of BlameLine
    pub fn new(lines: Vec<BlameLine<T>>) -> Self {
        Self { lines }
    }

    /// Get all lines in the result
    pub fn lines(&self) -> &[BlameLine<T>] {
        &self.lines
    }

    /// Get a specific line by index
    pub fn get_line(&self, index: usize) -> Option<&BlameLine<T>> {
        self.lines.get(index)
    }

    /// Get the total number of lines
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Iterate over all lines
    pub fn iter(&self) -> impl Iterator<Item = &BlameLine<T>> {
        self.lines.iter()
    }
}

impl<T> IntoIterator for BlameResult<T> {
    type Item = BlameLine<T>;
    type IntoIter = std::vec::IntoIter<BlameLine<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

/// Errors that can occur during blame operations
#[derive(Debug, thiserror::Error)]
pub enum BlameError {
    /// No revisions were provided
    #[error("no revisions provided")]
    EmptyRevisions,

    /// Invalid input data
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
