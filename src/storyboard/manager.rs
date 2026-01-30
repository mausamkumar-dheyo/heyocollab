//! StoryboardManager implementation with hybrid operations pattern.
//!
//! This module provides the main `StoryboardManager` struct that wraps an Automerge
//! document and provides:
//! - High-level operations via autosurgeon (hydrate/reconcile) for bulk updates
//! - Targeted O(1) updates via direct put operations for high-frequency fields
//! - Macro-generated CRUD for Character/Prop/Set with identical optimization paths

use automerge::{
    transaction::Transactable, AutoCommit, ChangeHash, ObjId, ReadDoc, ScalarValue, Value, ROOT,
};
use autosurgeon::{hydrate, reconcile};
use paste::paste;

use crate::error::{CollabError, CollabResult};
use crate::storyboard::model::*;

// =============================================================================
// ENTITY CRUD MACRO
// =============================================================================

/// Generates CRUD methods for an entity type with consistent O(1) optimization.
/// All setters follow identical optimization path: cache invalidate → get ObjId → put/delete
macro_rules! entity_crud {
    ($entity:ident, $collection:ident, $order:ident) => {
        paste! {
            /// Creates a new entity and appends it to the order list.
            pub fn [<create_ $collection:snake>](&mut self, id: &str, entity: $entity) -> CollabResult<()> {
                self.update_state(|state| {
                    let id_str = id.to_string();
                    state.processing_stages.$collection.insert(id_str.clone(), entity);
                    if !state.processing_stages.$order.contains(&id_str) {
                        state.processing_stages.$order.push(id_str);
                    }
                })
            }

            /// Gets an entity by ID.
            pub fn [<get_ $collection:snake>](&mut self, id: &str) -> CollabResult<Option<$entity>> {
                let state = self.get_state()?;
                Ok(state.processing_stages.$collection.get(id).cloned())
            }

            /// Deletes an entity by ID.
            pub fn [<delete_ $collection:snake>](&mut self, id: &str) -> CollabResult<()> {
                self.update_state(|state| {
                    state.processing_stages.$collection.remove(id);
                    state.processing_stages.$order.retain(|s| s != id);
                })
            }

            /// Sets the image field (O(1) targeted update).
            pub fn [<set_ $collection:snake _image>](&mut self, id: &str, image: Option<&str>) -> CollabResult<()> {
                self.set_entity_field_opt_str(
                    &["processing_stages", stringify!($collection), id],
                    "image",
                    image,
                )
            }

            /// Sets the generation_status field (O(1) targeted update).
            pub fn [<set_ $collection:snake _generation_status>](&mut self, id: &str, status: Option<&str>) -> CollabResult<()> {
                self.set_entity_field_opt_str(
                    &["processing_stages", stringify!($collection), id],
                    "generation_status",
                    status,
                )
            }

            /// Sets the description_status field (O(1) targeted update).
            pub fn [<set_ $collection:snake _description_status>](&mut self, id: &str, status: Option<&str>) -> CollabResult<()> {
                self.set_entity_field_opt_str(
                    &["processing_stages", stringify!($collection), id],
                    "description_status",
                    status,
                )
            }

            /// Appends to history (maintains max 20 entries).
            pub fn [<append_ $collection:snake _history>](&mut self, id: &str, entry: AssetHistory) -> CollabResult<()> {
                self.append_to_asset_history(
                    &["processing_stages", stringify!($collection), id],
                    entry,
                )
            }
        }
    };
}

// =============================================================================
// STORYBOARD MANAGER
// =============================================================================

/// The main collaborative document manager for storyboards.
///
/// Uses a hybrid approach:
/// - `update_state()` for bulk struct operations (uses hydrate/reconcile)
/// - `set_*_image()`, `set_*_generation_status()` for targeted O(1) updates
/// - `entity_crud!` macro generates consistent CRUD for Character/Prop/Set
pub struct StoryboardManager {
    doc: AutoCommit,
    /// Cached hydrated state - invalidated after direct document mutations.
    cached_state: Option<StoryboardRoot>,
}

impl StoryboardManager {
    // =========================================================================
    // INITIALIZATION
    // =========================================================================

    /// Creates a new empty StoryboardManager with an initialized document schema.
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        let root = StoryboardRoot::default();
        reconcile(&mut doc, &root).expect("Failed to initialize document");
        Self {
            doc,
            cached_state: Some(root),
        }
    }

    /// Creates a StoryboardManager from saved binary data.
    pub fn from_bytes(bytes: &[u8]) -> CollabResult<Self> {
        let doc = AutoCommit::load(bytes)?;
        Ok(Self {
            doc,
            cached_state: None,
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

    // =========================================================================
    // HIGH-LEVEL OPERATIONS (via Hydrate/Reconcile)
    // =========================================================================

    /// Hydrates the entire document state to Rust structs.
    pub fn get_state(&mut self) -> CollabResult<StoryboardRoot> {
        if let Some(ref cached) = self.cached_state {
            return Ok(cached.clone());
        }
        let state: StoryboardRoot = hydrate(&self.doc)?;
        self.cached_state = Some(state.clone());
        Ok(state)
    }

    /// Applies a function to mutate the state, then reconciles back to the document.
    pub fn update_state<F>(&mut self, f: F) -> CollabResult<()>
    where
        F: FnOnce(&mut StoryboardRoot),
    {
        let mut state = self.get_state()?;
        f(&mut state);
        reconcile(&mut self.doc, &state)?;
        self.cached_state = Some(state);
        Ok(())
    }

    // =========================================================================
    // ROOT METADATA OPERATIONS
    // =========================================================================

    /// Sets the storyboard title (O(1)).
    pub fn set_title(&mut self, title: &str) -> CollabResult<()> {
        self.cached_state = None;
        self.doc.put(&ROOT, "title", ScalarValue::Str(title.into()))?;
        Ok(())
    }

    /// Sets the storyboard description (O(1)).
    pub fn set_description(&mut self, description: &str) -> CollabResult<()> {
        self.cached_state = None;
        self.doc
            .put(&ROOT, "description", ScalarValue::Str(description.into()))?;
        Ok(())
    }

    /// Sets the storyboard status (O(1)).
    pub fn set_status(&mut self, status: &str) -> CollabResult<()> {
        self.cached_state = None;
        self.doc
            .put(&ROOT, "status", ScalarValue::Str(status.into()))?;
        Ok(())
    }

    /// Sets the current processing stage (O(1)).
    pub fn set_current_stage(&mut self, stage: &str) -> CollabResult<()> {
        self.cached_state = None;
        self.doc
            .put(&ROOT, "current_stage", ScalarValue::Str(stage.into()))?;
        Ok(())
    }

    /// Updates the last_updated timestamp (O(1)).
    pub fn touch_last_updated(&mut self, timestamp: i64) -> CollabResult<()> {
        self.cached_state = None;
        self.doc
            .put(&ROOT, "last_updated", ScalarValue::Int(timestamp))?;
        Ok(())
    }

    // =========================================================================
    // ENTITY CRUD (Macro-generated)
    // =========================================================================

    entity_crud!(Character, characters, character_order);
    entity_crud!(Prop, props, prop_order);
    entity_crud!(SetLocation, sets, set_order);

    // =========================================================================
    // SCENE OPERATIONS
    // =========================================================================

    /// Creates a new scene and appends it to the order list.
    pub fn create_scene(&mut self, id: &str, scene: Scene) -> CollabResult<()> {
        self.update_state(|state| {
            let id_str = id.to_string();
            state.scenes.insert(id_str.clone(), scene);
            if !state.scene_order.contains(&id_str) {
                state.scene_order.push(id_str);
            }
        })
    }

    /// Gets a scene by ID.
    pub fn get_scene(&mut self, id: &str) -> CollabResult<Option<Scene>> {
        let state = self.get_state()?;
        Ok(state.scenes.get(id).cloned())
    }

    /// Deletes a scene by ID.
    pub fn delete_scene(&mut self, id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            state.scenes.remove(id);
            state.scene_order.retain(|s| s != id);
        })
    }

    /// Reorders scenes.
    pub fn reorder_scenes(&mut self, new_order: Vec<String>) -> CollabResult<()> {
        self.update_state(|state| {
            state.scene_order = new_order;
        })
    }

    /// Sets a character look for a scene (by tag).
    pub fn set_character_look(
        &mut self,
        scene_id: &str,
        tag: &str,
        look: CharacterLook,
    ) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                scene.character_looks.insert(tag.to_string(), look);
            }
        })
    }

    /// Sets a character outfit for a scene (by tag).
    pub fn set_character_outfit(
        &mut self,
        scene_id: &str,
        tag: &str,
        outfit: CharacterOutfit,
    ) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                scene.character_outfits.insert(tag.to_string(), outfit);
            }
        })
    }

    /// Sets a looks_with_outfit for a scene (by tag).
    pub fn set_looks_with_outfit(
        &mut self,
        scene_id: &str,
        tag: &str,
        lwo: LooksWithOutfit,
    ) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                scene.looks_with_outfit.insert(tag.to_string(), lwo);
            }
        })
    }

    // =========================================================================
    // SHOT OPERATIONS
    // =========================================================================

    /// Creates a new shot in a scene and appends it to the shot order.
    pub fn create_shot(&mut self, scene_id: &str, shot_id: &str, shot: Shot) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                let shot_id_str = shot_id.to_string();
                scene.shots.insert(shot_id_str.clone(), shot);
                if !scene.shot_order.contains(&shot_id_str) {
                    scene.shot_order.push(shot_id_str);
                }
            }
        })
    }

    /// Gets a shot by ID from a scene.
    pub fn get_shot(&mut self, scene_id: &str, shot_id: &str) -> CollabResult<Option<Shot>> {
        let state = self.get_state()?;
        Ok(state
            .scenes
            .get(scene_id)
            .and_then(|s| s.shots.get(shot_id).cloned()))
    }

    /// Deletes a shot from a scene.
    pub fn delete_shot(&mut self, scene_id: &str, shot_id: &str) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                scene.shots.remove(shot_id);
                scene.shot_order.retain(|s| s != shot_id);
            }
        })
    }

    /// Reorders shots in a scene.
    pub fn reorder_shots(&mut self, scene_id: &str, new_order: Vec<String>) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                scene.shot_order = new_order;
            }
        })
    }

    /// Sets the shot image (O(1) targeted update).
    pub fn set_shot_image(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        image: Option<&str>,
    ) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "image", image)
    }

    /// Sets the shot generation status (O(1) targeted update).
    pub fn set_shot_generation_status(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        status: Option<&str>,
    ) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "generation_status", status)
    }

    /// Sets the shot image prompt (O(1) targeted update).
    pub fn set_shot_image_prompt(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        prompt: &str,
    ) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        self.doc
            .put(&shot_obj, "image_prompt", ScalarValue::Str(prompt.into()))?;
        Ok(())
    }

    /// Sets the shot ref_shot_id (O(1) targeted update).
    pub fn set_shot_ref_shot_id(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        ref_id: Option<i32>,
    ) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        match ref_id {
            Some(v) => self
                .doc
                .put(&shot_obj, "ref_shot_id", ScalarValue::Int(v as i64))?,
            None => {
                self.doc.delete(&shot_obj, "ref_shot_id")?;
            }
        }
        Ok(())
    }

    /// Appends to shot history (maintains max 20 entries).
    pub fn append_shot_history(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        entry: ShotHistory,
    ) -> CollabResult<()> {
        self.update_state(|state| {
            if let Some(scene) = state.scenes.get_mut(scene_id) {
                if let Some(shot) = scene.shots.get_mut(shot_id) {
                    // Prepend new entry
                    shot.history.insert(0, entry);
                    // Trim to max 20
                    if shot.history.len() > 20 {
                        shot.history.truncate(20);
                    }
                }
            }
        })
    }

    // =========================================================================
    // ENTITY FIELD SETTERS (Characters, Props, Sets)
    // =========================================================================

    /// Sets the entity name (O(1)).
    pub fn set_entity_name(&mut self, entity_type: &str, id: &str, name: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["processing_stages", entity_type, id])?;
        self.doc.put(&obj, "name", ScalarValue::Str(name.into()))?;
        Ok(())
    }

    /// Sets the entity description (O(1)).
    pub fn set_entity_description(&mut self, entity_type: &str, id: &str, description: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["processing_stages", entity_type, id])?;
        self.doc.put(&obj, "description", ScalarValue::Str(description.into()))?;
        Ok(())
    }

    /// Sets the entity tag (O(1)).
    pub fn set_entity_tag(&mut self, entity_type: &str, id: &str, tag: Option<&str>) -> CollabResult<()> {
        self.set_entity_field_opt_str(&["processing_stages", entity_type, id], "tag", tag)
    }

    /// Sets the entity image_prompt (O(1)).
    pub fn set_entity_image_prompt(&mut self, entity_type: &str, id: &str, prompt: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["processing_stages", entity_type, id])?;
        self.doc.put(&obj, "image_prompt", ScalarValue::Str(prompt.into()))?;
        Ok(())
    }

    /// Sets the entity caption (O(1)).
    pub fn set_entity_caption(&mut self, entity_type: &str, id: &str, caption: Option<&str>) -> CollabResult<()> {
        self.set_entity_field_opt_str(&["processing_stages", entity_type, id], "caption", caption)
    }

    /// Sets the entity enhanced flag (O(1)).
    pub fn set_entity_enhanced(&mut self, entity_type: &str, id: &str, enhanced: bool) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["processing_stages", entity_type, id])?;
        self.doc.put(&obj, "enhanced", ScalarValue::Boolean(enhanced))?;
        Ok(())
    }

    // =========================================================================
    // SCENE FIELD SETTERS
    // =========================================================================

    /// Sets the scene title (O(1)).
    pub fn set_scene_title(&mut self, scene_id: &str, title: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["scenes", scene_id])?;
        self.doc.put(&obj, "title", ScalarValue::Str(title.into()))?;
        Ok(())
    }

    /// Sets the scene synopsis (O(1)).
    pub fn set_scene_synopsis(&mut self, scene_id: &str, synopsis: Option<&str>) -> CollabResult<()> {
        self.set_scene_field_opt_str(scene_id, "synopsis", synopsis)
    }

    /// Sets the scene header (O(1)).
    pub fn set_scene_header(&mut self, scene_id: &str, header: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["scenes", scene_id])?;
        self.doc.put(&obj, "header", ScalarValue::Str(header.into()))?;
        Ok(())
    }

    /// Sets the scene content (O(1)).
    pub fn set_scene_content(&mut self, scene_id: &str, content: &str) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["scenes", scene_id])?;
        self.doc.put(&obj, "content", ScalarValue::Str(content.into()))?;
        Ok(())
    }

    /// Sets the scene raw_text (O(1)).
    pub fn set_scene_raw_text(&mut self, scene_id: &str, raw_text: Option<&str>) -> CollabResult<()> {
        self.set_scene_field_opt_str(scene_id, "raw_text", raw_text)
    }

    /// Sets the scene predicted_shots (O(1)).
    pub fn set_scene_predicted_shots(&mut self, scene_id: &str, predicted_shots: i64) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["scenes", scene_id])?;
        self.doc.put(&obj, "predicted_shots", ScalarValue::Int(predicted_shots))?;
        Ok(())
    }

    /// Sets the scene reasoning (O(1)).
    pub fn set_scene_reasoning(&mut self, scene_id: &str, reasoning: Option<&str>) -> CollabResult<()> {
        self.set_scene_field_opt_str(scene_id, "reasoning", reasoning)
    }

    /// Helper for scene optional string fields.
    fn set_scene_field_opt_str(&mut self, scene_id: &str, key: &str, value: Option<&str>) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(&["scenes", scene_id])?;
        match value {
            Some(v) => self.doc.put(&obj, key, ScalarValue::Str(v.into()))?,
            None => { self.doc.delete(&obj, key)?; }
        }
        Ok(())
    }

    // =========================================================================
    // ADDITIONAL SHOT FIELD SETTERS
    // =========================================================================

    /// Sets the shot visual_description (O(1)).
    pub fn set_shot_visual_description(&mut self, scene_id: &str, shot_id: &str, desc: &str) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        self.doc.put(&shot_obj, "visual_description", ScalarValue::Str(desc.into()))?;
        Ok(())
    }

    /// Sets the shot action (O(1)).
    pub fn set_shot_action(&mut self, scene_id: &str, shot_id: &str, action: Option<&str>) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "action", action)
    }

    /// Sets the shot camera (O(1)).
    pub fn set_shot_camera(&mut self, scene_id: &str, shot_id: &str, camera: Option<&str>) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "camera", camera)
    }

    /// Sets the shot environment (O(1)).
    pub fn set_shot_environment(&mut self, scene_id: &str, shot_id: &str, env: Option<&str>) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "environment", env)
    }

    /// Sets the shot subject (O(1)).
    pub fn set_shot_subject(&mut self, scene_id: &str, shot_id: &str, subject: Option<&str>) -> CollabResult<()> {
        self.set_shot_field_opt_str(scene_id, shot_id, "subject", subject)
    }

    /// Sets the shot size (O(1)).
    pub fn set_shot_size(&mut self, scene_id: &str, shot_id: &str, size: &str) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        self.doc.put(&shot_obj, "size", ScalarValue::Str(size.into()))?;
        Ok(())
    }

    /// Sets the shot angle (O(1)).
    pub fn set_shot_angle(&mut self, scene_id: &str, shot_id: &str, angle: &str) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        self.doc.put(&shot_obj, "angle", ScalarValue::Str(angle.into()))?;
        Ok(())
    }

    // =========================================================================
    // SYNC OPERATIONS
    // =========================================================================

    /// Merges another document into this one.
    pub fn merge(&mut self, other: &mut Self) -> CollabResult<()> {
        self.cached_state = None;
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
            bytes.extend(change.raw_bytes());
        }
        Some(bytes)
    }

    /// Applies sync message from peer.
    pub fn apply_sync_message(&mut self, msg: &[u8]) -> CollabResult<()> {
        self.cached_state = None;
        self.doc.load_incremental(msg)?;
        Ok(())
    }

    // =========================================================================
    // INTERNAL HELPERS - O(1) OPERATIONS
    // =========================================================================

    /// O(1) string field setter for entity types.
    fn set_entity_field_opt_str(
        &mut self,
        path: &[&str],
        key: &str,
        value: Option<&str>,
    ) -> CollabResult<()> {
        self.cached_state = None;
        let obj = self.get_obj_at_path(path)?;
        match value {
            Some(v) => self.doc.put(&obj, key, ScalarValue::Str(v.into()))?,
            None => {
                self.doc.delete(&obj, key)?;
            }
        }
        Ok(())
    }

    /// O(1) string field setter for shots.
    fn set_shot_field_opt_str(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        key: &str,
        value: Option<&str>,
    ) -> CollabResult<()> {
        self.cached_state = None;
        let shot_obj = self.get_shot_obj(scene_id, shot_id)?;
        match value {
            Some(v) => self.doc.put(&shot_obj, key, ScalarValue::Str(v.into()))?,
            None => {
                self.doc.delete(&shot_obj, key)?;
            }
        }
        Ok(())
    }

    /// Appends to asset history with max 20 limit.
    fn append_to_asset_history(&mut self, path: &[&str], entry: AssetHistory) -> CollabResult<()> {
        // For simplicity, use update_state. Could be optimized to direct list ops later.
        let path_vec: Vec<String> = path.iter().map(|s| s.to_string()).collect();

        self.update_state(move |state| {
            // Navigate to the entity based on path
            // Path format: ["processing_stages", "characters", "{id}"]
            if path_vec.len() >= 3 && path_vec[0] == "processing_stages" {
                let collection = &path_vec[1];
                let id = &path_vec[2];

                match collection.as_str() {
                    "characters" => {
                        if let Some(entity) = state.processing_stages.characters.get_mut(id) {
                            entity.history.insert(0, entry);
                            if entity.history.len() > 20 {
                                entity.history.truncate(20);
                            }
                        }
                    }
                    "props" => {
                        if let Some(entity) = state.processing_stages.props.get_mut(id) {
                            entity.history.insert(0, entry);
                            if entity.history.len() > 20 {
                                entity.history.truncate(20);
                            }
                        }
                    }
                    "sets" => {
                        if let Some(entity) = state.processing_stages.sets.get_mut(id) {
                            entity.history.insert(0, entry);
                            if entity.history.len() > 20 {
                                entity.history.truncate(20);
                            }
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    /// Gets ObjId at a path.
    fn get_obj_at_path(&self, path: &[&str]) -> CollabResult<ObjId> {
        let mut current = ROOT;
        for key in path {
            current = self.get_obj_at_key(&current, key)?;
        }
        Ok(current)
    }

    /// Gets ObjId for a shot.
    fn get_shot_obj(&self, scene_id: &str, shot_id: &str) -> CollabResult<ObjId> {
        let scenes_obj = self.get_obj_at_key(&ROOT, "scenes")?;
        let scene_obj = self.get_obj_at_key(&scenes_obj, scene_id)?;
        let shots_obj = self.get_obj_at_key(&scene_obj, "shots")?;
        self.get_obj_at_key(&shots_obj, shot_id)
    }

    /// Gets an object ID at a map key.
    fn get_obj_at_key(&self, parent: &ObjId, key: &str) -> CollabResult<ObjId> {
        match self.doc.get(parent, key) {
            Ok(Some((Value::Object(_), obj_id))) => Ok(obj_id),
            Ok(Some(_)) => Err(CollabError::schema_violation(format!(
                "'{}' is not an object",
                key
            ))),
            Ok(None) => Err(CollabError::field_not_found(key)),
            Err(e) => Err(CollabError::Automerge(e)),
        }
    }
}

impl Default for StoryboardManager {
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
        let mut manager = StoryboardManager::new();
        let state = manager.get_state().unwrap();
        assert!(state.scenes.is_empty());
        assert!(state.processing_stages.characters.is_empty());
    }

    #[test]
    fn test_create_character() {
        let mut manager = StoryboardManager::new();
        let character = Character::new("char-1", "John").with_tag("@john");

        manager.create_characters("char-1", character).unwrap();

        let retrieved = manager.get_characters("char-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "John");
    }

    #[test]
    fn test_create_prop() {
        let mut manager = StoryboardManager::new();
        let prop = Prop::new("prop-1", "Laptop").with_tag("@laptop");

        manager.create_props("prop-1", prop).unwrap();

        let retrieved = manager.get_props("prop-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Laptop");
    }

    #[test]
    fn test_create_set() {
        let mut manager = StoryboardManager::new();
        let set = SetLocation::new("set-1", "Office").with_tag("@office");

        manager.create_sets("set-1", set).unwrap();

        let retrieved = manager.get_sets("set-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Office");
    }

    #[test]
    fn test_targeted_image_update() {
        let mut manager = StoryboardManager::new();
        let character = Character::new("char-1", "John");
        manager.create_characters("char-1", character).unwrap();

        // O(1) update
        manager
            .set_characters_image("char-1", Some("https://example.com/john.png"))
            .unwrap();

        let retrieved = manager.get_characters("char-1").unwrap().unwrap();
        assert_eq!(
            retrieved.image,
            Some("https://example.com/john.png".to_string())
        );
    }

    #[test]
    fn test_targeted_status_update() {
        let mut manager = StoryboardManager::new();
        let character = Character::new("char-1", "John");
        manager.create_characters("char-1", character).unwrap();

        // O(1) update
        manager
            .set_characters_generation_status("char-1", Some("pending"))
            .unwrap();

        let retrieved = manager.get_characters("char-1").unwrap().unwrap();
        assert_eq!(retrieved.generation_status, Some("pending".to_string()));
    }

    #[test]
    fn test_create_scene_and_shot() {
        let mut manager = StoryboardManager::new();

        let scene = Scene::new("scene-1", 1).with_title("Opening");
        manager.create_scene("scene-1", scene).unwrap();

        let shot = Shot::new("shot-1", 1).with_image_prompt("Wide shot");
        manager.create_shot("scene-1", "shot-1", shot).unwrap();

        let retrieved = manager.get_shot("scene-1", "shot-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().image_prompt, "Wide shot");
    }

    #[test]
    fn test_shot_targeted_update() {
        let mut manager = StoryboardManager::new();

        let scene = Scene::new("scene-1", 1);
        manager.create_scene("scene-1", scene).unwrap();

        let shot = Shot::new("shot-1", 1);
        manager.create_shot("scene-1", "shot-1", shot).unwrap();

        // O(1) updates
        manager
            .set_shot_image("scene-1", "shot-1", Some("https://example.com/shot.png"))
            .unwrap();
        manager
            .set_shot_generation_status("scene-1", "shot-1", Some("completed"))
            .unwrap();
        manager
            .set_shot_ref_shot_id("scene-1", "shot-1", Some(-1))
            .unwrap();

        let retrieved = manager.get_shot("scene-1", "shot-1").unwrap().unwrap();
        assert_eq!(
            retrieved.image,
            Some("https://example.com/shot.png".to_string())
        );
        assert_eq!(retrieved.generation_status, Some("completed".to_string()));
        assert_eq!(retrieved.ref_shot_id, Some(-1));
    }

    #[test]
    fn test_history_append() {
        let mut manager = StoryboardManager::new();
        let character = Character::new("char-1", "John");
        manager.create_characters("char-1", character).unwrap();

        // Append multiple history entries
        for i in 0..25 {
            let entry =
                AssetHistory::new(format!("h-{}", i), format!("img-{}", i), format!("prompt-{}", i))
                    .with_timestamp(i as i64);
            manager.append_characters_history("char-1", entry).unwrap();
        }

        // Should be capped at 20
        let retrieved = manager.get_characters("char-1").unwrap().unwrap();
        assert_eq!(retrieved.history.len(), 20);

        // Most recent should be first
        assert_eq!(retrieved.history[0].id, "h-24");
    }

    #[test]
    fn test_save_and_load() {
        let mut manager = StoryboardManager::new();
        let character = Character::new("char-1", "John");
        manager.create_characters("char-1", character).unwrap();

        let bytes = manager.save();
        let mut loaded = StoryboardManager::from_bytes(&bytes).unwrap();

        let state = loaded.get_state().unwrap();
        assert!(state.processing_stages.characters.contains_key("char-1"));
    }

    #[test]
    fn test_merge_documents() {
        let mut base = StoryboardManager::new();
        let character = Character::new("base-char", "Base");
        base.create_characters("base-char", character).unwrap();

        let bytes = base.save();
        let mut client_a = StoryboardManager::from_bytes(&bytes).unwrap();
        let mut client_b = StoryboardManager::from_bytes(&bytes).unwrap();

        // Different changes
        let char_a = Character::new("char-a", "Alice");
        client_a.create_characters("char-a", char_a).unwrap();

        let char_b = Character::new("char-b", "Bob");
        client_b.create_characters("char-b", char_b).unwrap();

        // Merge
        client_a.merge(&mut client_b).unwrap();
        client_b.merge(&mut client_a).unwrap();

        // Both should have all characters
        let state_a = client_a.get_state().unwrap();
        let state_b = client_b.get_state().unwrap();

        assert_eq!(state_a.processing_stages.characters.len(), 3);
        assert_eq!(state_b.processing_stages.characters.len(), 3);
    }

    // =========================================================================
    // INTEGRATION TESTS - Real .automerge files
    // =========================================================================

    #[test]
    fn test_load_legend_automerge() {
        // Load the converted file
        let bytes = std::fs::read("src/bin/json2automerge/examples/legend.automerge")
            .expect("Failed to read legend.automerge");

        // Create manager from bytes
        let mut manager =
            StoryboardManager::from_bytes(&bytes).expect("Failed to load automerge");

        // Hydrate and verify
        let state = manager.get_state().expect("Failed to get state");

        // Verify metadata
        assert_eq!(state.id, "SUpXe7YkRm");
        assert_eq!(state.title, "legend");
        assert_eq!(state.status, "processing");

        // Verify counts
        assert_eq!(state.processing_stages.characters.len(), 2);
        assert_eq!(state.processing_stages.props.len(), 1);
        assert_eq!(state.processing_stages.sets.len(), 1);
        assert_eq!(state.scenes.len(), 1);

        // Verify scene has shots
        let scene = state.scenes.values().next().unwrap();
        assert_eq!(scene.shots.len(), 10);
    }

    #[test]
    fn test_legend_modify_and_save() {
        let bytes = std::fs::read("src/bin/json2automerge/examples/legend.automerge").unwrap();
        let mut manager = StoryboardManager::from_bytes(&bytes).unwrap();

        // Modify title
        manager
            .update_state(|state| {
                state.title = "legend - modified".to_string();
            })
            .unwrap();

        // Save and reload
        let new_bytes = manager.save();
        let mut manager2 = StoryboardManager::from_bytes(&new_bytes).unwrap();
        let state2 = manager2.get_state().unwrap();

        assert_eq!(state2.title, "legend - modified");

        // Original data still intact
        assert_eq!(state2.processing_stages.characters.len(), 2);
        assert_eq!(state2.scenes.len(), 1);
    }

    #[test]
    fn test_legend_character_access() {
        let bytes = std::fs::read("src/bin/json2automerge/examples/legend.automerge").unwrap();
        let mut manager = StoryboardManager::from_bytes(&bytes).unwrap();
        let state = manager.get_state().unwrap();

        // Verify character order is preserved
        assert_eq!(state.processing_stages.character_order.len(), 2);

        // Access characters by order
        for char_id in &state.processing_stages.character_order {
            let character = state.processing_stages.characters.get(char_id).unwrap();
            assert!(!character.name.is_empty());
            assert!(!character.id.is_empty());
        }
    }

    #[test]
    fn test_legend_shot_access() {
        let bytes = std::fs::read("src/bin/json2automerge/examples/legend.automerge").unwrap();
        let mut manager = StoryboardManager::from_bytes(&bytes).unwrap();
        let state = manager.get_state().unwrap();

        // Get first scene
        let scene_id = &state.scene_order[0];
        let scene = state.scenes.get(scene_id).unwrap();

        // Verify shot order
        assert_eq!(scene.shot_order.len(), 10);

        // Access shots by order
        for (i, shot_id) in scene.shot_order.iter().enumerate() {
            let shot = scene.shots.get(shot_id).unwrap();
            assert_eq!(shot.shot_number, (i + 1) as i32);
        }
    }
}
