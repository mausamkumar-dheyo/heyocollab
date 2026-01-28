//! Storyboard CRDT module for collaborative storyboard editing.
//!
//! This module provides:
//! - `model`: Data structures for storyboard (Character, Prop, SetLocation, Scene, Shot)
//! - `manager`: StoryboardManager with CRUD operations and O(1) targeted updates
//! - `wasm`: WASM bindings for browser usage (JsStoryboardManager)

pub mod manager;
pub mod model;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use manager::StoryboardManager;
pub use model::*;

#[cfg(feature = "wasm")]
pub use wasm::JsStoryboardManager;
