#[derive(Debug, Clone)]
pub struct BlameRevision<'a, T> {
    pub content: &'a str,
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
    pub fn new(lines: Vec<BlameLine<T>>) -> Self {
        Self { lines }
    }

    pub fn lines(&self) -> &[BlameLine<T>] {
        &self.lines
    }

    pub fn get_line(&self, index: usize) -> Option<&BlameLine<T>> {
        self.lines.get(index)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffAlgorithm {
    /// Myers diff algorithm (default, used by Git)
    Myers,
    /// Patience diff algorithm (better for code reorganization)
    Patience,
}

impl Default for DiffAlgorithm {
    fn default() -> Self {
        Self::Myers
    }
}

/// Options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlameOptions {
    /// Diff algorithm to use
    pub algorithm: DiffAlgorithm,
}

impl Default for BlameOptions {
    fn default() -> Self {
        Self {
            algorithm: DiffAlgorithm::default(),
        }
    }
}

/// Errors
#[derive(Debug, thiserror::Error)]
pub enum BlameError {
    /// No revisions were provided
    #[error("no revisions provided")]
    EmptyRevisions,

    /// Invalid input data
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
