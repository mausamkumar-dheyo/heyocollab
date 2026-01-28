//! HeyoCollab - Real-time collaborative document manager for AI generation sequences.
//!
//! This crate provides a developer-friendly implementation using Autosurgeon
//! for automatic CRDT serialization with a local-first architecture:
//!
//! - **Local-first text editing**: Text fields stay in UI state until Generate is clicked
//! - **Generation-level sync**: Each user creates their own generation nodes with final values
//! - **No conflicts**: Users work on separate generations, eliminating merge conflicts
//!
//! # Example
//!
//! ```rust
//! use heyocollab::{SequenceManager, GenerationNode, GenerationSettings};
//!
//! // Create a new collaborative document
//! let mut manager = SequenceManager::new();
//!
//! // User edits locally in UI (not shown - stays in React state)
//! // When user clicks "Generate", create a node with final values
//! let node = GenerationNode::new("gen-1", "t2i")
//!     .with_prompt("Cinematic, a beautiful sunset over the ocean")
//!     .with_settings(
//!         GenerationSettings::new()
//!             .with_seed(42)
//!             .with_cfg(7.5)
//!     );
//!
//! // Add to document and sync
//! manager.create_and_append("gen-1", node).unwrap();
//!
//! // Save for sync (one operation, not per-character)
//! let bytes = manager.save();
//! ```

pub mod error;

// Sequence module
pub mod sequence;

// Re-exports for convenience
pub use error::{CollabError, CollabResult};
pub use sequence::{DocumentRoot, GenerationNode, GenerationSettings, OutputAsset, SequenceManager};

#[cfg(feature = "wasm")]
pub use sequence::JsSequenceManager;

// Storyboard module (only compiled when storyboard feature enabled)
#[cfg(feature = "storyboard")]
pub mod storyboard;

#[cfg(feature = "storyboard")]
pub use storyboard::{StoryboardManager, StoryboardRoot};

#[cfg(all(feature = "wasm", feature = "storyboard"))]
pub use storyboard::wasm::JsStoryboardManager;
