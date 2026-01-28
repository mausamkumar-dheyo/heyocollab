//! WASM bindings for sequence module.
//!
//! This module provides JavaScript-friendly wrappers around the core
//! SequenceManager and related types for use in browser environments.

use js_sys::{Array, Uint8Array};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, Serializer};
use wasm_bindgen::prelude::*;

use crate::error::CollabError;
use super::manager::SequenceManager;
use super::model::{GenerationNode, OutputAsset};

/// Serialize a value to JsValue with HashMaps as plain JS objects (not Map).
fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&Serializer::new().serialize_maps_as_objects(true))
}

// =============================================================================
// ERROR CONVERSION
// =============================================================================

impl From<CollabError> for JsValue {
    fn from(err: CollabError) -> JsValue {
        JsValue::from_str(&err.to_string())
    }
}

/// Helper macro for Result conversion
macro_rules! js_result {
    ($expr:expr) => {
        $expr.map_err(|e: CollabError| JsValue::from(e))
    };
}

// =============================================================================
// MAIN WRAPPER TYPE
// =============================================================================

/// JavaScript-friendly wrapper around SequenceManager.
///
/// This provides a collaborative document manager for AI generation sequences
/// that can be used from JavaScript/TypeScript in the browser.
#[wasm_bindgen]
pub struct JsSequenceManager {
    inner: SequenceManager,
}

#[wasm_bindgen]
impl JsSequenceManager {
    /// Creates a new empty sequence manager.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const manager = new JsSequenceManager();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsSequenceManager {
        JsSequenceManager {
            inner: SequenceManager::new()
        }
    }

    /// Loads from binary bytes (Uint8Array).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const bytes = new Uint8Array([...]); // Saved document bytes
    /// const manager = JsSequenceManager.fromBytes(bytes);
    /// ```
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<JsSequenceManager, JsValue> {
        let inner = js_result!(SequenceManager::from_bytes(bytes))?;
        Ok(JsSequenceManager { inner })
    }

    /// Saves to binary bytes (returns Uint8Array).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const bytes = manager.toBytes();
    /// // Save bytes to localStorage, IndexedDB, or send to server
    /// ```
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&mut self) -> Uint8Array {
        let bytes = self.inner.save();
        Uint8Array::from(&bytes[..])
    }

    /// Gets the full document state as a JavaScript object.
    ///
    /// Returns an object with `sequence_order` (array of IDs) and
    /// `generations` (map of ID -> GenerationNode).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const state = manager.getState();
    /// console.log(state.sequence_order); // ['gen-1', 'gen-2']
    /// console.log(state.generations['gen-1'].prompt); // "A beautiful sunset"
    /// ```
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&mut self) -> Result<JsValue, JsValue> {
        let state = js_result!(self.inner.get_state())?;
        Ok(to_js_value(&state)?)
    }

    /// Gets the actor ID for this document instance.
    ///
    /// Each manager instance has a unique actor ID used to track changes.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const actorId = manager.actorId();
    /// console.log(actorId); // "a1b2c3d4..." (64-char hex string)
    /// ```
    #[wasm_bindgen(js_name = actorId)]
    pub fn actor_id(&self) -> String {
        self.inner.actor_id()
    }

    /// Gets the current heads (for sync protocol).
    ///
    /// Heads represent the current state of the document and are used
    /// to determine what changes need to be synced.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const heads = manager.getHeads();
    /// // Send heads to server to request sync
    /// ```
    #[wasm_bindgen(js_name = getHeads)]
    pub fn get_heads(&mut self) -> Array {
        let heads = self.inner.get_heads();
        let array = Array::new();
        for head in heads {
            array.push(&JsValue::from_str(&head.to_string()));
        }
        array
    }
}

// =============================================================================
// NODE MANAGEMENT METHODS
// =============================================================================

#[wasm_bindgen]
impl JsSequenceManager {
    /// Creates a node and appends it to the sequence.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the node
    /// * `node` - GenerationNode as JavaScript object with fields:
    ///   - `id`: string
    ///   - `type_`: string (e.g., "t2i", "i2v")
    ///   - `status`: string (e.g., "pending", "processing", "completed")
    ///   - `title`: string
    ///   - `prompt`: string
    ///   - `negative_prompt`: string
    ///   - `notes`: string
    ///   - `settings`: object with optional fields (seed, cfg, num_steps, etc.)
    ///   - `outputs`: array of OutputAsset objects
    ///   - `metadata`: string (JSON)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.createAndAppend('gen-1', {
    ///   id: 'gen-1',
    ///   type_: 't2i',
    ///   status: 'pending',
    ///   title: 'My Image',
    ///   prompt: 'A beautiful sunset',
    ///   negative_prompt: '',
    ///   notes: '',
    ///   settings: { seed: 42, cfg: 7.5 },
    ///   outputs: [],
    ///   metadata: ''
    /// });
    /// ```
    #[wasm_bindgen(js_name = createAndAppend)]
    pub fn create_and_append(&mut self, id: &str, node: JsValue) -> Result<(), JsValue> {
        let node: GenerationNode = from_value(node)?;
        js_result!(self.inner.create_and_append(id, node))?;
        Ok(())
    }

    /// Gets a node by ID, returns null if not found.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const node = manager.getNode('gen-1');
    /// if (node) {
    ///   console.log(node.prompt);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getNode)]
    pub fn get_node(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let node = js_result!(self.inner.get_node(id))?;
        match node {
            Some(n) => Ok(to_js_value(&n)?),
            None => Ok(JsValue::NULL)
        }
    }

    /// Deletes a node by ID.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.deleteNode('gen-1');
    /// ```
    #[wasm_bindgen(js_name = deleteNode)]
    pub fn delete_node(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.delete_node(id))?;
        Ok(())
    }
}

// =============================================================================
// ORDER MANAGEMENT METHODS
// =============================================================================

#[wasm_bindgen]
impl JsSequenceManager {
    /// Appends a node ID to the sequence order.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.appendGeneration('gen-2');
    /// ```
    #[wasm_bindgen(js_name = appendGeneration)]
    pub fn append_generation(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.append_generation(id))?;
        Ok(())
    }

    /// Removes a node ID from the sequence order.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.removeFromOrder('gen-1');
    /// ```
    #[wasm_bindgen(js_name = removeFromOrder)]
    pub fn remove_from_order(&mut self, id: &str) -> Result<(), JsValue> {
        js_result!(self.inner.remove_from_order(id))?;
        Ok(())
    }

    /// Moves a generation from one position to another.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.moveGeneration(0, 2); // Move first item to third position
    /// ```
    #[wasm_bindgen(js_name = moveGeneration)]
    pub fn move_generation(&mut self, from: usize, to: usize) -> Result<(), JsValue> {
        js_result!(self.inner.move_generation(from, to))?;
        Ok(())
    }

    /// Gets the current sequence order as an array of IDs.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const order = manager.getOrder();
    /// console.log(order); // ['gen-1', 'gen-2', 'gen-3']
    /// ```
    #[wasm_bindgen(js_name = getOrder)]
    pub fn get_order(&mut self) -> Result<Array, JsValue> {
        let order = js_result!(self.inner.get_order())?;
        let array = Array::new();
        for id in order {
            array.push(&JsValue::from_str(&id));
        }
        Ok(array)
    }
}

// =============================================================================
// SETTINGS METHODS
// =============================================================================

#[wasm_bindgen]
impl JsSequenceManager {
    /// Sets the seed setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingSeed('gen-1', 42);
    /// manager.setSettingSeed('gen-1', null); // Clear setting
    /// ```
    #[wasm_bindgen(js_name = setSettingSeed)]
    pub fn set_setting_seed(&mut self, node_id: &str, seed: Option<f64>) -> Result<(), JsValue> {
        let seed_i64 = seed.map(|s| s as i64);
        js_result!(self.inner.set_setting_seed(node_id, seed_i64))?;
        Ok(())
    }

    /// Sets the CFG setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingCfg('gen-1', 7.5);
    /// ```
    #[wasm_bindgen(js_name = setSettingCfg)]
    pub fn set_setting_cfg(&mut self, node_id: &str, cfg: Option<f64>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_cfg(node_id, cfg))?;
        Ok(())
    }

    /// Sets the num_steps setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingNumSteps('gen-1', 30);
    /// ```
    #[wasm_bindgen(js_name = setSettingNumSteps)]
    pub fn set_setting_num_steps(&mut self, node_id: &str, steps: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_num_steps(node_id, steps))?;
        Ok(())
    }

    /// Sets the model setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingModel('gen-1', 'stable-diffusion-xl');
    /// ```
    #[wasm_bindgen(js_name = setSettingModel)]
    pub fn set_setting_model(&mut self, node_id: &str, model: Option<String>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_model(node_id, model.as_deref()))?;
        Ok(())
    }

    /// Sets the resolution setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingResolution('gen-1', 1080);
    /// ```
    #[wasm_bindgen(js_name = setSettingResolution)]
    pub fn set_setting_resolution(&mut self, node_id: &str, resolution: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_resolution(node_id, resolution))?;
        Ok(())
    }

    /// Sets the width setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingWidth('gen-1', 1024);
    /// ```
    #[wasm_bindgen(js_name = setSettingWidth)]
    pub fn set_setting_width(&mut self, node_id: &str, width: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_width(node_id, width))?;
        Ok(())
    }

    /// Sets the height setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingHeight('gen-1', 1024);
    /// ```
    #[wasm_bindgen(js_name = setSettingHeight)]
    pub fn set_setting_height(&mut self, node_id: &str, height: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_height(node_id, height))?;
        Ok(())
    }

    /// Sets the duration setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingDuration('gen-1', 5); // 5 seconds
    /// ```
    #[wasm_bindgen(js_name = setSettingDuration)]
    pub fn set_setting_duration(&mut self, node_id: &str, duration: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_duration(node_id, duration))?;
        Ok(())
    }

    /// Sets the FPS setting (pass null to clear).
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setSettingFps('gen-1', 30);
    /// ```
    #[wasm_bindgen(js_name = setSettingFps)]
    pub fn set_setting_fps(&mut self, node_id: &str, fps: Option<i32>) -> Result<(), JsValue> {
        js_result!(self.inner.set_setting_fps(node_id, fps))?;
        Ok(())
    }
}

// =============================================================================
// STATUS AND OUTPUT METHODS
// =============================================================================

#[wasm_bindgen]
impl JsSequenceManager {
    /// Sets the status of a generation node.
    ///
    /// Common statuses: "pending", "processing", "completed", "failed", "cancelled"
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.setStatus('gen-1', 'processing');
    /// ```
    #[wasm_bindgen(js_name = setStatus)]
    pub fn set_status(&mut self, node_id: &str, status: &str) -> Result<(), JsValue> {
        js_result!(self.inner.set_status(node_id, status))?;
        Ok(())
    }

    /// Adds an output asset to a generation node.
    ///
    /// # Arguments
    /// * `node_id` - ID of the generation node
    /// * `output` - OutputAsset object with fields:
    ///   - `url`: string
    ///   - `seed`: number (optional)
    ///   - `is_selected`: boolean
    ///
    /// # Example (JavaScript)
    /// ```js
    /// manager.addOutput('gen-1', {
    ///   url: 'https://cdn.example.com/image.png',
    ///   seed: 42,
    ///   is_selected: true
    /// });
    /// ```
    #[wasm_bindgen(js_name = addOutput)]
    pub fn add_output(&mut self, node_id: &str, output: JsValue) -> Result<(), JsValue> {
        let output: OutputAsset = from_value(output)?;
        js_result!(self.inner.add_output(node_id, output))?;
        Ok(())
    }
}

// =============================================================================
// SYNC PROTOCOL METHODS
// =============================================================================

#[wasm_bindgen]
impl JsSequenceManager {
    /// Merges another manager's changes into this one.
    ///
    /// This is typically used for local merging. For network sync,
    /// use generateSyncMessage/applySyncMessage instead.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const manager1 = new JsSequenceManager();
    /// const manager2 = new JsSequenceManager();
    /// // ... make changes to manager2
    /// manager1.merge(manager2); // Merge manager2's changes into manager1
    /// ```
    pub fn merge(&mut self, other: &mut JsSequenceManager) -> Result<(), JsValue> {
        js_result!(self.inner.merge(&mut other.inner))?;
        Ok(())
    }

    /// Generates a sync message for changes since their heads.
    ///
    /// Returns a Uint8Array containing the sync message, or null if no changes.
    ///
    /// # Arguments
    /// * `their_heads` - Array of head strings from the remote peer (currently unused, pass [])
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const syncMsg = manager.generateSyncMessage([]);
    /// if (syncMsg) {
    ///   // Convert to base64 and send over WebSocket
    ///   const base64 = btoa(String.fromCharCode(...syncMsg));
    ///   ws.send(JSON.stringify({ type: 'sync', message: base64 }));
    /// }
    /// ```
    #[wasm_bindgen(js_name = generateSyncMessage)]
    pub fn generate_sync_message(&mut self, _their_heads: Array) -> Result<JsValue, JsValue> {
        // TODO: Parse their_heads array and convert to Vec<ChangeHash>
        // For now, generate sync message from empty heads (full document)
        match self.inner.generate_sync_message(&[]) {
            Some(bytes) => Ok(Uint8Array::from(&bytes[..]).into()),
            None => Ok(JsValue::NULL)
        }
    }

    /// Applies a sync message from a peer.
    ///
    /// # Arguments
    /// * `msg` - Sync message bytes (Uint8Array)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// // Received base64-encoded sync message from server
    /// const bytes = new Uint8Array(atob(data.message).split('').map(c => c.charCodeAt(0)));
    /// manager.applySyncMessage(bytes);
    ///
    /// // Update UI with new state
    /// const state = manager.getState();
    /// ```
    #[wasm_bindgen(js_name = applySyncMessage)]
    pub fn apply_sync_message(&mut self, msg: &[u8]) -> Result<(), JsValue> {
        js_result!(self.inner.apply_sync_message(msg))?;
        Ok(())
    }
}
