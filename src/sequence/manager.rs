//! Core SequenceManager implementation with hybrid operations pattern.
//!
//! This module provides the main `SequenceManager` struct that wraps an Automerge
//! document and provides:
//! - High-level operations via autosurgeon (hydrate/reconcile) for bulk updates
//! - Targeted settings updates via direct put operations (O(1) instead of O(N))

use automerge::{
    transaction::Transactable, AutoCommit, ChangeHash, ObjId, ReadDoc, ScalarValue, Value,
    ROOT,
};
use autosurgeon::{hydrate, reconcile};

use crate::error::{CollabError, CollabResult};
use super::model::{DocumentRoot, GenerationNode, GenerationSettings, OutputAsset};

/// The main collaborative document manager for AI generation sequences.
///
/// Uses a hybrid approach:
/// - `update_state()` for bulk struct operations (uses hydrate/reconcile)
/// - `splice_text()` for character-level text editing (direct Automerge API)
/// - `set_setting_*()` for targeted settings updates (direct put, O(1))
///
/// # Caching Strategy
///
/// - `cached_state`: Full DocumentRoot, invalidated on any direct mutation
/// - `cached_generations_obj`: ObjId of the "generations" map, invalidated on load/merge
pub struct SequenceManager {
    doc: AutoCommit,
    /// Cached hydrated state - invalidated after direct document mutations.
    cached_state: Option<DocumentRoot>,
    /// Cached ObjId for the "generations" map - saves 2 lookups per operation.
    /// Invalidated on from_bytes() and merge().
    cached_generations_obj: Option<ObjId>,
}

impl SequenceManager {
    // =========================================================================
    // INITIALIZATION
    // =========================================================================

    /// Creates a new empty SequenceManager with an initialized document schema.
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        let root = DocumentRoot::default();
        reconcile(&mut doc, &root).expect("Failed to initialize document");
        Self {
            doc,
            cached_state: Some(root),
            cached_generations_obj: None, // Will be lazily populated
        }
    }

    /// Creates a SequenceManager from saved binary data.
    pub fn from_bytes(bytes: &[u8]) -> CollabResult<Self> {
        let doc = AutoCommit::load(bytes)?;
        Ok(Self {
            doc,
            cached_state: None,
            cached_generations_obj: None, // Must re-discover after load
        })
    }

    /// Saves the document to binary format.
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Returns the current heads (for sync protocol).
    pub fn get_heads(&mut self) -> Vec<ChangeHash> {
        self.doc.get_heads()
    }

    /// Gets the actor ID for this document instance.
    pub fn actor_id(&self) -> String {
        self.doc.get_actor().to_hex_string()
    }

    /// Invalidates all caches. Call after any operation that might change document structure.
    fn invalidate_all_caches(&mut self) {
        self.cached_state = None;
        self.cached_generations_obj = None;
    }

    // =========================================================================
    // HIGH-LEVEL OPERATIONS (via Hydrate/Reconcile)
    // =========================================================================

    /// Hydrates the entire document state to Rust structs.
    pub fn get_state(&mut self) -> CollabResult<DocumentRoot> {
        if let Some(ref cached) = self.cached_state {
            return Ok(cached.clone());
        }
        let state: DocumentRoot = hydrate(&self.doc)?;
        self.cached_state = Some(state.clone());
        Ok(state)
    }

    /// Applies a function to mutate the state, then reconciles back to the document.
    /// Use this for bulk updates where text performance isn't critical.
    pub fn update_state<F>(&mut self, f: F) -> CollabResult<()>
    where
        F: FnOnce(&mut DocumentRoot),
    {
        let mut state = self.get_state()?;
        f(&mut state);
        reconcile(&mut self.doc, &state)?;
        self.cached_state = Some(state);
        // Note: Don't invalidate cached_generations_obj - reconcile doesn't change ObjIds
        Ok(())
    }

    /// Creates a new generation node.
    pub fn create_node(&mut self, id: &str, node: GenerationNode) -> CollabResult<()> {
        self.update_state(|state| {
            state.generations.insert(id.to_string(), node);
        })
    }

    /// Appends a generation ID to the sequence order.
    pub fn append_generation(&mut self, id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            let id_str = id.to_string();
            if !state.sequence_order.contains(&id_str) {
                state.sequence_order.push(id_str);
            }
        })
    }

    /// Creates a node and appends it to the sequence order in one operation.
    pub fn create_and_append(&mut self, id: &str, node: GenerationNode) -> CollabResult<()> {
        self.update_state(|state| {
            let id_str = id.to_string();
            state.generations.insert(id_str.clone(), node);
            if !state.sequence_order.contains(&id_str) {
                state.sequence_order.push(id_str);
            }
        })
    }

    /// Gets a node by ID.
    pub fn get_node(&mut self, id: &str) -> CollabResult<Option<GenerationNode>> {
        let state = self.get_state()?;
        Ok(state.generations.get(id).cloned())
    }

    /// Updates a node's fields.
    pub fn update_node<F>(&mut self, id: &str, f: F) -> CollabResult<()>
    where
        F: FnOnce(&mut GenerationNode),
    {
        self.update_state(|state| {
            if let Some(node) = state.generations.get_mut(id) {
                f(node);
            }
        })
    }

    /// Updates a node's settings (full reconcile version).
    /// For single-field updates, prefer `set_setting_*` methods.
    pub fn update_settings<F>(&mut self, id: &str, f: F) -> CollabResult<()>
    where
        F: FnOnce(&mut GenerationSettings),
    {
        self.update_state(|state| {
            if let Some(node) = state.generations.get_mut(id) {
                f(&mut node.settings);
            }
        })
    }

    /// Adds an output to a node.
    pub fn add_output(&mut self, node_id: &str, output: OutputAsset) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(node) = state.generations.get_mut(node_id) {
                node.outputs.push(output);
            }
        })
    }

    /// Removes a node from the document.
    pub fn delete_node(&mut self, id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            state.generations.remove(id);
            state.sequence_order.retain(|s| s != id);
        })
    }

    /// Removes a generation from the sequence order (by ID).
    pub fn remove_from_order(&mut self, id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            state.sequence_order.retain(|s| s != id);
        })
    }

    /// Inserts a generation at a specific position in the sequence order.
    pub fn insert_at_position(&mut self, index: usize, id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            let id_str = id.to_string();
            if index <= state.sequence_order.len() && !state.sequence_order.contains(&id_str) {
                state.sequence_order.insert(index, id_str);
            }
        })
    }

    /// Moves a generation from one position to another.
    pub fn move_generation(&mut self, from: usize, to: usize) -> CollabResult<()> {
        self.update_state(|state| {
            let len = state.sequence_order.len();
            if from < len && to <= len && from != to {
                let id = state.sequence_order.remove(from);
                let adjusted_to = if from < to { to - 1 } else { to };
                state.sequence_order.insert(adjusted_to, id);
            }
        })
    }

    /// Returns the ordered list of generation IDs.
    pub fn get_order(&mut self) -> CollabResult<Vec<String>> {
        let state = self.get_state()?;
        Ok(state.sequence_order.clone())
    }

    // =========================================================================
    // TARGETED SETTINGS UPDATES (Direct put, O(1))
    // =========================================================================

    /// Sets a single setting value directly, bypassing full reconcile.
    /// This is O(1) instead of O(N) where N is document size.
    fn set_setting_value(
        &mut self,
        node_id: &str,
        key: &str,
        value: ScalarValue,
    ) -> CollabResult<()> {
        self.cached_state = None; // Invalidate state cache
        let settings_obj = self.get_settings_obj(node_id)?;
        self.doc.put(&settings_obj, key, value)?;
        Ok(())
    }

    /// Clears a setting (for Option::None).
    /// OPTIMIZATION: Use delete() instead of put(Null) - saves space.
    fn set_setting_null(&mut self, node_id: &str, key: &str) -> CollabResult<()> {
        self.cached_state = None;
        let settings_obj = self.get_settings_obj(node_id)?;
        self.doc.delete(&settings_obj, key)?;
        Ok(())
    }

    /// Sets the seed setting directly (O(1)).
    pub fn set_setting_seed(&mut self, node_id: &str, seed: Option<i64>) -> CollabResult<()> {
        match seed {
            Some(v) => self.set_setting_value(node_id, "seed", ScalarValue::Int(v)),
            None => self.set_setting_null(node_id, "seed"),
        }
    }

    /// Sets the cfg (guidance scale) setting directly (O(1)).
    pub fn set_setting_cfg(&mut self, node_id: &str, cfg: Option<f64>) -> CollabResult<()> {
        match cfg {
            Some(v) => self.set_setting_value(node_id, "cfg", ScalarValue::F64(v)),
            None => self.set_setting_null(node_id, "cfg"),
        }
    }

    /// Sets the num_steps setting directly (O(1)).
    pub fn set_setting_num_steps(&mut self, node_id: &str, steps: Option<i32>) -> CollabResult<()> {
        match steps {
            Some(v) => self.set_setting_value(node_id, "num_steps", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "num_steps"),
        }
    }

    /// Sets the model setting directly (O(1)).
    pub fn set_setting_model(&mut self, node_id: &str, model: Option<&str>) -> CollabResult<()> {
        match model {
            Some(v) => self.set_setting_value(node_id, "model", ScalarValue::Str(v.into())),
            None => self.set_setting_null(node_id, "model"),
        }
    }

    /// Sets the resolution setting directly (O(1)).
    pub fn set_setting_resolution(
        &mut self,
        node_id: &str,
        resolution: Option<i32>,
    ) -> CollabResult<()> {
        match resolution {
            Some(v) => self.set_setting_value(node_id, "resolution", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "resolution"),
        }
    }

    /// Sets the width setting directly (O(1)).
    pub fn set_setting_width(&mut self, node_id: &str, width: Option<i32>) -> CollabResult<()> {
        match width {
            Some(v) => self.set_setting_value(node_id, "width", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "width"),
        }
    }

    /// Sets the height setting directly (O(1)).
    pub fn set_setting_height(&mut self, node_id: &str, height: Option<i32>) -> CollabResult<()> {
        match height {
            Some(v) => self.set_setting_value(node_id, "height", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "height"),
        }
    }

    /// Sets the duration setting directly (O(1)).
    pub fn set_setting_duration(
        &mut self,
        node_id: &str,
        duration: Option<i32>,
    ) -> CollabResult<()> {
        match duration {
            Some(v) => self.set_setting_value(node_id, "duration", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "duration"),
        }
    }

    /// Sets the fps setting directly (O(1)).
    pub fn set_setting_fps(&mut self, node_id: &str, fps: Option<i32>) -> CollabResult<()> {
        match fps {
            Some(v) => self.set_setting_value(node_id, "fps", ScalarValue::Int(v as i64)),
            None => self.set_setting_null(node_id, "fps"),
        }
    }

    /// Sets the node status directly (O(1)).
    pub fn set_status(&mut self, node_id: &str, status: &str) -> CollabResult<()> {
        self.cached_state = None;
        let node_obj = self.get_node_obj(node_id)?;
        self.doc
            .put(&node_obj, "status", ScalarValue::Str(status.into()))?;
        Ok(())
    }

    // =========================================================================
    // LOW-LEVEL TEXT OPERATIONS (Direct Automerge API for performance)
    // =========================================================================

    // =========================================================================
    // SYNC OPERATIONS
    // =========================================================================

    /// Merges another document into this one.
    pub fn merge(&mut self, other: &mut Self) -> CollabResult<()> {
        self.invalidate_all_caches(); // Must invalidate topology cache on merge
        self.doc.merge(&mut other.doc)?;
        Ok(())
    }

    /// Generates sync message for incremental sync.
    /// Returns None if there are no changes since their_heads.
    pub fn generate_sync_message(&mut self, their_heads: &[ChangeHash]) -> Option<Vec<u8>> {
        let changes = self.doc.get_changes(their_heads);
        if changes.is_empty() {
            return None;
        }
        let mut bytes = Vec::new();
        for change in changes {
            bytes.extend_from_slice(change.raw_bytes());
        }
        Some(bytes)
    }

    /// Applies sync message from peer.
    pub fn apply_sync_message(&mut self, msg: &[u8]) -> CollabResult<()> {
        self.invalidate_all_caches(); // Must invalidate topology cache on sync
        self.doc.load_incremental(msg)?;
        Ok(())
    }

    // =========================================================================
    // COMPRESSION METHODS
    // =========================================================================

    // =========================================================================
    // INTERNAL HELPERS - WITH TOPOLOGY CACHING
    // =========================================================================

    /// Gets the cached "generations" map ObjId, or discovers it.
    fn get_generations_obj(&mut self) -> CollabResult<ObjId> {
        if let Some(ref obj) = self.cached_generations_obj {
            return Ok(obj.clone());
        }
        let obj = self.get_obj_at_key(&ROOT, "generations")?;
        self.cached_generations_obj = Some(obj.clone());
        Ok(obj)
    }

    /// Gets a node's ObjId using the cached generations map.
    fn get_node_obj(&mut self, node_id: &str) -> CollabResult<ObjId> {
        let gens_obj = self.get_generations_obj()?;
        self.get_obj_at_key(&gens_obj, node_id)
    }

    /// Gets the settings ObjId for a node.
    fn get_settings_obj(&mut self, node_id: &str) -> CollabResult<ObjId> {
        let node_obj = self.get_node_obj(node_id)?;
        self.get_obj_at_key(&node_obj, "settings")
    }

    /// Gets an object ID at a map key.
    fn get_obj_at_key(&self, parent: &ObjId, key: &str) -> CollabResult<ObjId> {
        match self.doc.get(parent, key) {
            Ok(Some((Value::Object(_), obj_id))) => Ok(obj_id),
            Ok(Some(_)) => Err(CollabError::schema_violation(format!(
                "'{}' is not an object",
                key
            ))),
            Ok(None) => {
                if key.len() == 36 {
                    // Likely a UUID - node not found
                    Err(CollabError::node_not_found(key))
                } else {
                    Err(CollabError::field_not_found(key))
                }
            }
            Err(e) => Err(CollabError::Automerge(e)),
        }
    }
}

impl Default for SequenceManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let mut manager = SequenceManager::new();
        let state = manager.get_state().unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn test_create_and_append() {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test-id", "t2i").with_prompt("A beautiful sunset");

        manager.create_and_append("test-id", node).unwrap();

        let state = manager.get_state().unwrap();
        assert_eq!(state.len(), 1);
        assert_eq!(state.sequence_order.len(), 1);
        assert_eq!(state.sequence_order[0], "test-id");
    }

    #[test]
    fn test_save_and_load() {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test-id", "t2i");
        manager.create_and_append("test-id", node).unwrap();

        let bytes = manager.save();
        let mut loaded = SequenceManager::from_bytes(&bytes).unwrap();

        let state = loaded.get_state().unwrap();
        assert_eq!(state.len(), 1);
        assert!(state.generations.contains_key("test-id"));
    }

    #[test]
    fn test_update_settings() {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test-id", "t2i");
        manager.create_and_append("test-id", node).unwrap();

        manager
            .update_settings("test-id", |settings| {
                settings.seed = Some(42);
                settings.cfg = Some(7.5);
            })
            .unwrap();

        let node = manager.get_node("test-id").unwrap().unwrap();
        assert_eq!(node.settings.seed, Some(42));
        assert_eq!(node.settings.cfg, Some(7.5));
    }

    #[test]
    fn test_targeted_settings_update() {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test-id", "t2i");
        manager.create_and_append("test-id", node).unwrap();

        // Use direct O(1) setting updates
        manager.set_setting_seed("test-id", Some(123)).unwrap();
        manager.set_setting_cfg("test-id", Some(8.5)).unwrap();
        manager
            .set_setting_model("test-id", Some("sdxl-turbo"))
            .unwrap();
        manager.set_setting_width("test-id", Some(1024)).unwrap();
        manager.set_setting_height("test-id", Some(768)).unwrap();

        let node = manager.get_node("test-id").unwrap().unwrap();
        assert_eq!(node.settings.seed, Some(123));
        assert_eq!(node.settings.cfg, Some(8.5));
        assert_eq!(node.settings.model, Some("sdxl-turbo".to_string()));
        assert_eq!(node.settings.width, Some(1024));
        assert_eq!(node.settings.height, Some(768));

        // Test deletion
        manager.set_setting_seed("test-id", None).unwrap();
        let node = manager.get_node("test-id").unwrap().unwrap();
        assert_eq!(node.settings.seed, None);
    }

    #[test]
    fn test_set_status() {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test-id", "t2i");
        manager.create_and_append("test-id", node).unwrap();

        manager.set_status("test-id", "processing").unwrap();
        let node = manager.get_node("test-id").unwrap().unwrap();
        assert_eq!(node.status, "processing");

        manager.set_status("test-id", "completed").unwrap();
        let node = manager.get_node("test-id").unwrap().unwrap();
        assert_eq!(node.status, "completed");
    }

    #[test]
    fn test_merge_documents() {
        // Create base document
        let mut base = SequenceManager::new();
        let node = GenerationNode::new("base-node", "t2i");
        base.create_and_append("base-node", node).unwrap();

        // Fork to two clients
        let bytes = base.save();
        let mut client_a = SequenceManager::from_bytes(&bytes).unwrap();
        let mut client_b = SequenceManager::from_bytes(&bytes).unwrap();

        // Make different changes
        let node_a = GenerationNode::new("node-a", "t2i");
        client_a.create_and_append("node-a", node_a).unwrap();

        let node_b = GenerationNode::new("node-b", "i2v");
        client_b.create_and_append("node-b", node_b).unwrap();

        // Merge
        client_a.merge(&mut client_b).unwrap();
        client_b.merge(&mut client_a).unwrap();

        // Both should have all nodes
        let state_a = client_a.get_state().unwrap();
        let state_b = client_b.get_state().unwrap();

        assert_eq!(state_a.len(), 3);
        assert_eq!(state_b.len(), 3);
        assert!(state_a.generations.contains_key("base-node"));
        assert!(state_a.generations.contains_key("node-a"));
        assert!(state_a.generations.contains_key("node-b"));
    }

    #[test]
    fn test_string_text_fields() {
        let mut manager = SequenceManager::new();

        let node = GenerationNode::new("id", "t2i")
            .with_prompt("Test prompt")
            .with_title("Test title")
            .with_negative_prompt("bad quality")
            .with_notes("Some notes");

        manager.create_and_append("id", node).unwrap();

        let state = manager.get_state().unwrap();
        let node = state.generations.get("id").unwrap();

        assert_eq!(node.prompt_str(), "Test prompt");
        assert_eq!(node.title_str(), "Test title");
        assert_eq!(node.negative_prompt_str(), "bad quality");
        assert_eq!(node.notes_str(), "Some notes");
    }

    #[test]
    fn test_inspect_bloat() {
        println!("\n=== AUTOSURGEON BLOAT INSPECTION ===\n");

        let mut manager = SequenceManager::new();

        // Create just 1 node with default values
        manager
            .create_node("test-1", GenerationNode::new("test-1", "t2i"))
            .unwrap();

        // 1. Binary size
        let binary = manager.save();
        println!("Binary Size: {} bytes ({:.2} KB)", binary.len(), binary.len() as f64 / 1024.0);

        // 2. JSON Structure (Logical View)
        let json_struct = serde_json::to_string_pretty(&manager.get_state().unwrap().generations.get("test-1").unwrap().to_json_value()).unwrap();
        println!("\n--- Logical Structure (1 node) ---\n{}", json_struct);

        // 3. Change count and operation breakdown
        let changes = manager.doc.get_changes(&[]);
        println!("\n--- Automerge Changes: {} total ---", changes.len());

        let mut total_ops = 0;
        for (i, change) in changes.iter().enumerate() {
            let ops_in_change = change.len();
            total_ops += ops_in_change;
            println!(
                "  Change {}: {} bytes raw, {} ops",
                i,
                change.raw_bytes().len(),
                ops_in_change
            );
        }
        println!("\nTotal Operations: {}", total_ops);
        println!("Bytes per Op: {:.2}", binary.len() as f64 / total_ops as f64);

        // 4. Show the actual document skeleton
        println!("\n--- Document Skeleton (What Autosurgeon Creates) ---");
        fn print_obj(doc: &AutoCommit, obj: &ObjId, indent: usize) {
            let prefix = "  ".repeat(indent);
            for item in doc.map_range(obj, ..) {
                match &item.value {
                    Value::Object(obj_type) => {
                        println!("{}{}: {:?}", prefix, item.key, obj_type);
                        if let Ok(Some((_, child_id))) = doc.get(obj, item.key) {
                            print_obj(doc, &child_id, indent + 1);
                        }
                    }
                    Value::Scalar(s) => {
                        println!("{}{}: {:?}", prefix, item.key, s.as_ref());
                    }
                }
            }
        }
        print_obj(&manager.doc, &ROOT, 0);

        // 5. Breakdown by category
        println!("\n--- Skeleton Cost Breakdown ---");
        println!("Root level:");
        println!("  - sequence_order: List (1 op)");
        println!("  - generations: Map (1 op)");
        println!("\nPer GenerationNode:");
        println!("  - Node map entry (1 op)");
        println!("  - id: String (1 op)");
        println!("  - type_: String (1 op)");
        println!("  - status: String (1 op)");
        println!("  - title: String (1 op) - local-first!");
        println!("  - prompt: String (1 op) - local-first!");
        println!("  - negative_prompt: String (1 op) - local-first!");
        println!("  - notes: String (1 op) - local-first!");
        println!("  - settings: Map (1 op) - sparse now, 0 children if all None");
        println!("  - outputs: List (1 op)");
        println!("  - metadata: String (1 op)");
        println!("\nTotal per node: ~9 ops (was 13 with Text objects, 22 with non-sparse settings)");
    }
}
