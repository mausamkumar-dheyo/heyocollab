//! Error types for the collaborative sequence manager.

use thiserror::Error;

/// Result type alias for collab operations.
pub type CollabResult<T> = Result<T, CollabError>;

/// Errors that can occur during collaborative operations.
#[derive(Error, Debug)]
pub enum CollabError {
    /// Automerge error during document operations.
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    /// Autosurgeon hydration error.
    #[error("Hydration error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),

    /// Autosurgeon reconcile error.
    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),

    /// Node not found in the document.
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Field not found in a node.
    #[error("Field not found: {0}")]
    FieldNotFound(String),

    /// Invalid text splice operation.
    #[error("Invalid splice: index {index} + delete {delete} exceeds text length {length}")]
    InvalidSplice {
        index: usize,
        delete: usize,
        length: usize,
    },

    /// Schema violation - document structure is invalid.
    #[error("Schema violation: {0}")]
    SchemaViolation(String),

    /// Index out of bounds for list operations.
    #[error("Index {index} out of bounds for list of length {length}")]
    IndexOutOfBounds { index: usize, length: usize },

    /// Invalid UUID string.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl CollabError {
    /// Creates a NodeNotFound error.
    pub fn node_not_found(id: impl Into<String>) -> Self {
        Self::NodeNotFound(id.into())
    }

    /// Creates a FieldNotFound error.
    pub fn field_not_found(field: impl Into<String>) -> Self {
        Self::FieldNotFound(field.into())
    }

    /// Creates an InvalidSplice error.
    pub fn invalid_splice(index: usize, delete: usize, length: usize) -> Self {
        Self::InvalidSplice {
            index,
            delete,
            length,
        }
    }

    /// Creates a SchemaViolation error.
    pub fn schema_violation(msg: impl Into<String>) -> Self {
        Self::SchemaViolation(msg.into())
    }

    /// Creates an IndexOutOfBounds error.
    pub fn index_out_of_bounds(index: usize, length: usize) -> Self {
        Self::IndexOutOfBounds { index, length }
    }

    /// Creates an InvalidUuid error.
    pub fn invalid_uuid(uuid: impl Into<String>) -> Self {
        Self::InvalidUuid(uuid.into())
    }

    /// Creates a Serialization error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
}
