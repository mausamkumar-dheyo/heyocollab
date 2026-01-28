//! Sequence document module.
//!
//! Provides the CRDT-based collaborative document manager for AI generation sequences.

pub mod model;
pub mod manager;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-exports for convenience
pub use model::{DocumentRoot, GenerationNode, GenerationSettings, OutputAsset};
pub use manager::SequenceManager;

#[cfg(feature = "wasm")]
pub use wasm::JsSequenceManager;
