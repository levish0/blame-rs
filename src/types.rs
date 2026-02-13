use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct BlameRevision<'a, T> {
    pub content: &'a str,
    pub metadata: Rc<T>,
}

#[derive(Debug, Clone)]
pub struct BlameLine<'a, T> {
    pub line_number: usize,
    pub content: &'a str,
    pub revision_metadata: Rc<T>,
}

/// The result of a blame operation, containing all lines with their origin information
#[derive(Debug, Clone)]
pub struct BlameResult<'a, T> {
    lines: Vec<BlameLine<'a, T>>,
}

impl<'a, T> BlameResult<'a, T> {
    pub fn new(lines: Vec<BlameLine<'a, T>>) -> Self {
        Self { lines }
    }

    pub fn lines(&self) -> &[BlameLine<'a, T>] {
        &self.lines
    }

    pub fn get_line(&self, index: usize) -> Option<&BlameLine<'a, T>> {
        self.lines.get(index)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlameLine<'a, T>> {
        self.lines.iter()
    }
}

impl<'a, T> IntoIterator for BlameResult<'a, T> {
    type Item = BlameLine<'a, T>;
    type IntoIter = std::vec::IntoIter<BlameLine<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffAlgorithm {
    /// Myers diff algorithm (default)
    #[default]
    Myers,
    /// Patience diff algorithm (better for code reorganization)
    Patience,
}

/// Options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlameOptions {
    /// Diff algorithm to use
    pub algorithm: DiffAlgorithm,
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
