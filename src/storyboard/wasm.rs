//! WASM bindings for storyboard module.
//!
//! This module provides JavaScript-friendly wrappers around the
//! StoryboardManager for use in browser environments.

use js_sys::{Array, Uint8Array};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, Serializer};
use wasm_bindgen::prelude::*;

use crate::storyboard::manager::StoryboardManager;
use crate::storyboard::model::*;
use crate::CollabError;

/// Serialize a value to JsValue with HashMaps as plain JS objects (not Map).
fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&Serializer::new().serialize_maps_as_objects(true))
}

// =============================================================================
// ERROR CONVERSION
// =============================================================================

/// Helper macro for Result conversion
macro_rules! js_result {
    ($expr:expr) => {
        $expr.map_err(|e: CollabError| JsValue::from_str(&e.to_string()))
    };
}

// =============================================================================
// MAIN WRAPPER TYPE
// =============================================================================

/// JavaScript-friendly wrapper around StoryboardManager.
///
/// This provides a collaborative document manager for storyboards
/// that can be used from JavaScript/TypeScript in the browser.
#[wasm_bindgen]
pub struct JsStoryboardManager {
    inner: StoryboardManager,
}

#[wasm_bindgen]
impl JsStoryboardManager {
    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Creates a new empty storyboard manager.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const manager = new JsStoryboardManager();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsStoryboardManager {
        JsStoryboardManager {
            inner: StoryboardManager::new(),
        }
    }

    /// Loads from binary bytes (Uint8Array).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const bytes = new Uint8Array([...]);
    /// const manager = JsStoryboardManager.fromBytes(bytes);
    /// ```
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<JsStoryboardManager, JsValue> {
        let inner = js_result!(StoryboardManager::from_bytes(bytes))?;
        Ok(JsStoryboardManager { inner })
    }

    /// Saves to binary bytes (returns Uint8Array).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const bytes = manager.toBytes();
    /// ```
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&mut self) -> Uint8Array {
        let bytes = self.inner.save();
        Uint8Array::from(&bytes[..])
    }

    /// Gets the actor ID for this document instance.
    #[wasm_bindgen(js_name = actorId)]
    pub fn actor_id(&self) -> String {
        self.inner.actor_id()
    }

    /// Gets the current heads (for sync protocol).
    #[wasm_bindgen(js_name = getHeads)]
    pub fn get_heads(&mut self) -> Array {
        let heads = self.inner.get_heads();
        heads
            .into_iter()
            .map(|h| JsValue::from_str(&h.to_string()))
            .collect()
    }

    // =========================================================================
    // STATE ACCESS
    // =========================================================================

    /// Gets the full document state as a JavaScript object.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const state = manager.getState();
    /// console.log(state.scenes);
    /// console.log(state.processing_stages.characters);
    /// ```
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&mut self) -> Result<JsValue, JsValue> {
        let state = js_result!(self.inner.get_state())?;
        Ok(to_js_value(&state)?)
    }

    // =========================================================================
    // ROOT OPERATIONS
    // =========================================================================

    /// Sets the storyboard title.
    #[wasm_bindgen(js_name = setTitle)]
    pub fn set_title(&mut self, title: &str) -> Result<(), JsValue> {
        js_result!(self.inner.set_title(title))
    }

    /// Sets the storyboard description.
    #[wasm_bindgen(js_name = setDescription)]
    pub fn set_description(&mut self, description: &str) -> Result<(), JsValue> {
        js_result!(self.inner.set_description(description))
    }

    /// Sets the storyboard status.
    #[wasm_bindgen(js_name = setStatus)]
    pub fn set_status(&mut self, status: &str) -> Result<(), JsValue> {
        js_result!(self.inner.set_status(status))
    }

    /// Sets the current processing stage.
    #[wasm_bindgen(js_name = setCurrentStage)]
    pub fn set_current_stage(&mut self, stage: &str) -> Result<(), JsValue> {
        js_result!(self.inner.set_current_stage(stage))
    }

    /// Updates the last_updated timestamp.
    #[wasm_bindgen(js_name = touchLastUpdated)]
    pub fn touch_last_updated(&mut self, timestamp: i64) -> Result<(), JsValue> {
        js_result!(self.inner.touch_last_updated(timestamp))
    }

    // =========================================================================
    // CHARACTER OPERATIONS
    // =========================================================================

    /// Creates a new character.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.createCharacter('char-1', {
    ///   id: 'char-1',
    ///   name: 'John',
    ///   description: 'A tall man',
    ///   tag: '@john'
    /// });
    /// ```
    #[wasm_bindgen(js_name = createCharacter)]
    pub fn create_character(&mut self, id: &str, character: JsValue) -> Result<(), JsValue> {
        let character: Character = from_value(character)?;
        js_result!(self.inner.create_characters(id, character))
    }

    /// Gets a character by ID.
    #[wasm_bindgen(js_name = getCharacter)]
    pub fn get_character(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let character = js_result!(self.inner.get_characters(id))?;
        Ok(to_js_value(&character)?)
    }

    /// Deletes a character by ID.
    #[wasm_bindgen(js_name = deleteCharacter)]
    pub fn delete_character(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_characters(id))
    }

    /// Sets the character image (O(1)).
    #[wasm_bindgen(js_name = setCharacterImage)]
    pub fn set_character_image(&mut self, id: &str, image: Option<String>) -> Result<(), JsValue> {
        js_result!(self.inner.set_characters_image(id, image.as_deref()))
    }

    /// Sets the character generation status (O(1)).
    #[wasm_bindgen(js_name = setCharacterGenerationStatus)]
    pub fn set_character_generation_status(
        &mut self,
        id: &str,
        status: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_characters_generation_status(id, status.as_deref()))
    }

    /// Sets the character description status (O(1)).
    #[wasm_bindgen(js_name = setCharacterDescriptionStatus)]
    pub fn set_character_description_status(
        &mut self,
        id: &str,
        status: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_characters_description_status(id, status.as_deref()))
    }

    /// Appends to character history.
    #[wasm_bindgen(js_name = appendCharacterHistory)]
    pub fn append_character_history(&mut self, id: &str, entry: JsValue) -> Result<(), JsValue> {
        let entry: AssetHistory = from_value(entry)?;
        js_result!(self.inner.append_characters_history(id, entry))
    }

    // =========================================================================
    // PROP OPERATIONS
    // =========================================================================

    /// Creates a new prop.
    #[wasm_bindgen(js_name = createProp)]
    pub fn create_prop(&mut self, id: &str, prop: JsValue) -> Result<(), JsValue> {
        let prop: Prop = from_value(prop)?;
        js_result!(self.inner.create_props(id, prop))
    }

    /// Gets a prop by ID.
    #[wasm_bindgen(js_name = getProp)]
    pub fn get_prop(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let prop = js_result!(self.inner.get_props(id))?;
        Ok(to_js_value(&prop)?)
    }

    /// Deletes a prop by ID.
    #[wasm_bindgen(js_name = deleteProp)]
    pub fn delete_prop(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_props(id))
    }

    /// Sets the prop image (O(1)).
    #[wasm_bindgen(js_name = setPropImage)]
    pub fn set_prop_image(&mut self, id: &str, image: Option<String>) -> Result<(), JsValue> {
        js_result!(self.inner.set_props_image(id, image.as_deref()))
    }

    /// Sets the prop generation status (O(1)).
    #[wasm_bindgen(js_name = setPropGenerationStatus)]
    pub fn set_prop_generation_status(
        &mut self,
        id: &str,
        status: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_props_generation_status(id, status.as_deref()))
    }

    /// Appends to prop history.
    #[wasm_bindgen(js_name = appendPropHistory)]
    pub fn append_prop_history(&mut self, id: &str, entry: JsValue) -> Result<(), JsValue> {
        let entry: AssetHistory = from_value(entry)?;
        js_result!(self.inner.append_props_history(id, entry))
    }

    // =========================================================================
    // SET OPERATIONS
    // =========================================================================

    /// Creates a new set/location.
    #[wasm_bindgen(js_name = createSet)]
    pub fn create_set(&mut self, id: &str, set_loc: JsValue) -> Result<(), JsValue> {
        let set_loc: SetLocation = from_value(set_loc)?;
        js_result!(self.inner.create_sets(id, set_loc))
    }

    /// Gets a set by ID.
    #[wasm_bindgen(js_name = getSet)]
    pub fn get_set(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let set_loc = js_result!(self.inner.get_sets(id))?;
        Ok(to_js_value(&set_loc)?)
    }

    /// Deletes a set by ID.
    #[wasm_bindgen(js_name = deleteSet)]
    pub fn delete_set(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_sets(id))
    }

    /// Sets the set image (O(1)).
    #[wasm_bindgen(js_name = setSetImage)]
    pub fn set_set_image(&mut self, id: &str, image: Option<String>) -> Result<(), JsValue> {
        js_result!(self.inner.set_sets_image(id, image.as_deref()))
    }

    /// Sets the set generation status (O(1)).
    #[wasm_bindgen(js_name = setSetGenerationStatus)]
    pub fn set_set_generation_status(
        &mut self,
        id: &str,
        status: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_sets_generation_status(id, status.as_deref()))
    }

    /// Appends to set history.
    #[wasm_bindgen(js_name = appendSetHistory)]
    pub fn append_set_history(&mut self, id: &str, entry: JsValue) -> Result<(), JsValue> {
        let entry: AssetHistory = from_value(entry)?;
        js_result!(self.inner.append_sets_history(id, entry))
    }

    // =========================================================================
    // SCENE OPERATIONS
    // =========================================================================

    /// Creates a new scene.
    #[wasm_bindgen(js_name = createScene)]
    pub fn create_scene(&mut self, id: &str, scene: JsValue) -> Result<(), JsValue> {
        let scene: Scene = from_value(scene)?;
        js_result!(self.inner.create_scene(id, scene))
    }

    /// Gets a scene by ID.
    #[wasm_bindgen(js_name = getScene)]
    pub fn get_scene(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let scene = js_result!(self.inner.get_scene(id))?;
        Ok(to_js_value(&scene)?)
    }

    /// Deletes a scene by ID.
    #[wasm_bindgen(js_name = deleteScene)]
    pub fn delete_scene(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_scene(id))
    }

    /// Reorders scenes.
    #[wasm_bindgen(js_name = reorderScenes)]
    pub fn reorder_scenes(&mut self, new_order: Array) -> Result<(), JsValue> {
        let order: Vec<String> = new_order
            .iter()
            .filter_map(|v| v.as_string())
            .collect();
        js_result!(self.inner.reorder_scenes(order))
    }

    /// Sets a character look for a scene.
    #[wasm_bindgen(js_name = setCharacterLook)]
    pub fn set_character_look(
        &mut self,
        scene_id: &str,
        tag: &str,
        look: JsValue,
    ) -> Result<(), JsValue> {
        let look: CharacterLook = from_value(look)?;
        js_result!(self.inner.set_character_look(scene_id, tag, look))
    }

    /// Sets a character outfit for a scene.
    #[wasm_bindgen(js_name = setCharacterOutfit)]
    pub fn set_character_outfit(
        &mut self,
        scene_id: &str,
        tag: &str,
        outfit: JsValue,
    ) -> Result<(), JsValue> {
        let outfit: CharacterOutfit = from_value(outfit)?;
        js_result!(self.inner.set_character_outfit(scene_id, tag, outfit))
    }

    /// Sets a looks_with_outfit for a scene.
    #[wasm_bindgen(js_name = setLooksWithOutfit)]
    pub fn set_looks_with_outfit(
        &mut self,
        scene_id: &str,
        tag: &str,
        lwo: JsValue,
    ) -> Result<(), JsValue> {
        let lwo: LooksWithOutfit = from_value(lwo)?;
        js_result!(self.inner.set_looks_with_outfit(scene_id, tag, lwo))
    }

    // =========================================================================
    // SHOT OPERATIONS
    // =========================================================================

    /// Creates a new shot in a scene.
    #[wasm_bindgen(js_name = createShot)]
    pub fn create_shot(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        shot: JsValue,
    ) -> Result<(), JsValue> {
        let shot: Shot = from_value(shot)?;
        js_result!(self.inner.create_shot(scene_id, shot_id, shot))
    }

    /// Gets a shot by ID from a scene.
    #[wasm_bindgen(js_name = getShot)]
    pub fn get_shot(&mut self, scene_id: &str, shot_id: &str) -> Result<JsValue, JsValue> {
        let shot = js_result!(self.inner.get_shot(scene_id, shot_id))?;
        Ok(to_js_value(&shot)?)
    }

    /// Deletes a shot from a scene.
    #[wasm_bindgen(js_name = deleteShot)]
    pub fn delete_shot(&mut self, scene_id: &str, shot_id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_shot(scene_id, shot_id))
    }

    /// Reorders shots in a scene.
    #[wasm_bindgen(js_name = reorderShots)]
    pub fn reorder_shots(&mut self, scene_id: &str, new_order: Array) -> Result<(), JsValue> {
        let order: Vec<String> = new_order
            .iter()
            .filter_map(|v| v.as_string())
            .collect();
        js_result!(self.inner.reorder_shots(scene_id, order))
    }

    /// Sets the shot image (O(1)).
    #[wasm_bindgen(js_name = setShotImage)]
    pub fn set_shot_image(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        image: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_shot_image(scene_id, shot_id, image.as_deref()))
    }

    /// Sets the shot generation status (O(1)).
    #[wasm_bindgen(js_name = setShotGenerationStatus)]
    pub fn set_shot_generation_status(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        status: Option<String>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_shot_generation_status(scene_id, shot_id, status.as_deref()))
    }

    /// Sets the shot image prompt (O(1)).
    #[wasm_bindgen(js_name = setShotImagePrompt)]
    pub fn set_shot_image_prompt(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        prompt: &str,
    ) -> Result<(), JsValue> {
        js_result!(self.inner.set_shot_image_prompt(scene_id, shot_id, prompt))
    }

    /// Sets the shot ref_shot_id (O(1)).
    #[wasm_bindgen(js_name = setShotRefShotId)]
    pub fn set_shot_ref_shot_id(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        ref_id: Option<i32>,
    ) -> Result<(), JsValue> {
        js_result!(self
            .inner
            .set_shot_ref_shot_id(scene_id, shot_id, ref_id))
    }

    /// Appends to shot history.
    #[wasm_bindgen(js_name = appendShotHistory)]
    pub fn append_shot_history(
        &mut self,
        scene_id: &str,
        shot_id: &str,
        entry: JsValue,
    ) -> Result<(), JsValue> {
        let entry: ShotHistory = from_value(entry)?;
        js_result!(self.inner.append_shot_history(scene_id, shot_id, entry))
    }

    // =========================================================================
    // SYNC OPERATIONS
    // =========================================================================

    /// Merges another manager's changes into this one.
    #[wasm_bindgen]
    pub fn merge(&mut self, other: &mut JsStoryboardManager) -> Result<(), JsValue> {
        js_result!(self.inner.merge(&mut other.inner))
    }

    /// Generates a sync message for changes since their heads.
    #[wasm_bindgen(js_name = generateSyncMessage)]
    pub fn generate_sync_message(&mut self, _their_heads: Array) -> Result<JsValue, JsValue> {
        // TODO: Parse their_heads properly
        let msg = self.inner.generate_sync_message(&[]);
        match msg {
            Some(bytes) => Ok(Uint8Array::from(&bytes[..]).into()),
            None => Ok(JsValue::NULL),
        }
    }

    /// Applies a sync message from a peer.
    #[wasm_bindgen(js_name = applySyncMessage)]
    pub fn apply_sync_message(&mut self, msg: &[u8]) -> Result<(), JsValue> {
        js_result!(self.inner.apply_sync_message(msg))
    }
}

impl Default for JsStoryboardManager {
    fn default() -> Self {
        Self::new()
    }
}
