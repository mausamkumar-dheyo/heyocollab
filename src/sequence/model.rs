//! Data models for the collaborative sequence manager.
//!
//! These structs use autosurgeon derives for automatic CRDT serialization.

use automerge::{ScalarValue, Value};
use autosurgeon::reconcile::{MapReconciler, NoKey};
use autosurgeon::{Hydrate, HydrateError, ReadDoc, Reconcile, Reconciler};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// DOCUMENT ROOT
// =============================================================================

/// Root document structure for a collaborative sequence.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct DocumentRoot {
    /// Ordered list of generation UUIDs (as strings).
    pub sequence_order: Vec<String>,

    /// Map of UUID string -> GenerationNode.
    pub generations: HashMap<String, GenerationNode>,
}

impl DocumentRoot {
    /// Creates a new empty document root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of generations.
    pub fn len(&self) -> usize {
        self.generations.len()
    }

    /// Returns true if there are no generations.
    pub fn is_empty(&self) -> bool {
        self.generations.is_empty()
    }
}

// =============================================================================
// GENERATION NODE
// =============================================================================

/// A single generation node with all collaborative fields.
///
/// Text fields (title, prompt, negative_prompt, notes) are local-first Strings.
/// They are edited locally in the UI and only synced when the user clicks Generate.
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct GenerationNode {
    /// Unique identifier (stored for convenience, key in map is authoritative).
    pub id: String,

    /// Generation type: "t2i", "i2v", "text-to-image", etc.
    pub type_: String,

    /// Status: "pending", "processing", "completed", "failed", "queued", "cancelled".
    pub status: String,

    /// Text fields - local-first, synced on Generate click.
    pub title: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub notes: String,

    /// Generation settings (nested struct).
    pub settings: GenerationSettings,

    /// List of output assets.
    pub outputs: Vec<OutputAsset>,

    /// Extensible metadata as JSON string (blob approach).
    pub metadata: String,
}

impl GenerationNode {
    /// Creates a new GenerationNode with the given id and type.
    pub fn new(id: impl Into<String>, type_: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            type_: type_.into(),
            status: "pending".to_string(),
            title: String::new(),
            prompt: String::new(),
            negative_prompt: String::new(),
            notes: String::new(),
            settings: GenerationSettings::default(),
            outputs: Vec::new(),
            metadata: String::new(),
        }
    }

    /// Builder: Set status.
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Builder: Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: Set prompt.
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Builder: Set negative prompt.
    pub fn with_negative_prompt(mut self, negative_prompt: impl Into<String>) -> Self {
        self.negative_prompt = negative_prompt.into();
        self
    }

    /// Builder: Set notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    /// Builder: Set settings.
    pub fn with_settings(mut self, settings: GenerationSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Builder: Add an output.
    pub fn with_output(mut self, output: OutputAsset) -> Self {
        self.outputs.push(output);
        self
    }

    /// Builder: Set metadata as JSON string.
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = metadata.into();
        self
    }

    /// Gets the title as a string slice.
    pub fn title_str(&self) -> &str {
        &self.title
    }

    /// Gets the prompt as a string slice.
    pub fn prompt_str(&self) -> &str {
        &self.prompt
    }

    /// Gets the negative prompt as a string slice.
    pub fn negative_prompt_str(&self) -> &str {
        &self.negative_prompt
    }

    /// Gets the notes as a string slice.
    pub fn notes_str(&self) -> &str {
        &self.notes
    }

    /// Converts to a JSON-serializable representation.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "type_": self.type_,
            "status": self.status,
            "title": self.title,
            "prompt": self.prompt,
            "negative_prompt": self.negative_prompt,
            "notes": self.notes,
            "settings": self.settings,
            "outputs": self.outputs,
            "metadata": self.metadata,
        })
    }
}

impl Default for GenerationNode {
    fn default() -> Self {
        Self::new("", "")
    }
}

// =============================================================================
// GENERATION SETTINGS
// =============================================================================

/// Settings for AI generation.
/// Note: Reconcile and Hydrate are implemented manually for sparse serialization.
/// - Reconcile: Only writes Some() fields, deletes None fields
/// - Hydrate: Treats missing keys as None (instead of erroring)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GenerationSettings {
    /// Random seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Classifier-free guidance scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg: Option<f64>,

    /// Number of inference steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_steps: Option<i32>,

    /// Model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Resolution preset (e.g., 720, 1080).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<i32>,

    /// Duration in seconds (for video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,

    /// Output width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,

    /// Output height in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,

    /// Frames per second (for video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<i32>,
}

impl GenerationSettings {
    /// Creates new empty settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: Set seed.
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Builder: Set CFG scale.
    pub fn with_cfg(mut self, cfg: f64) -> Self {
        self.cfg = Some(cfg);
        self
    }

    /// Builder: Set number of steps.
    pub fn with_num_steps(mut self, steps: i32) -> Self {
        self.num_steps = Some(steps);
        self
    }

    /// Builder: Set model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Builder: Set resolution.
    pub fn with_resolution(mut self, resolution: i32) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Builder: Set duration.
    pub fn with_duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Builder: Set width.
    pub fn with_width(mut self, width: i32) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: Set height.
    pub fn with_height(mut self, height: i32) -> Self {
        self.height = Some(height);
        self
    }

    /// Builder: Set FPS.
    pub fn with_fps(mut self, fps: i32) -> Self {
        self.fps = Some(fps);
        self
    }
}

/// Sparse Reconcile implementation: only writes Some() fields, deletes None fields.
/// This eliminates the 9 extra null operations per node that the derive macro creates.
impl Reconcile for GenerationSettings {
    type Key<'a> = NoKey;

    fn reconcile<R: Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        let mut m = reconciler.map()?;

        // Helper: put if Some, delete if None (clears stale keys)
        macro_rules! reconcile_opt {
            ($field:expr, $key:literal) => {
                match $field {
                    Some(v) => m.put($key, v)?,
                    None => {
                        let _ = m.delete($key);
                    }
                }
            };
        }

        reconcile_opt!(self.seed, "seed");
        reconcile_opt!(self.cfg, "cfg");
        reconcile_opt!(self.num_steps, "num_steps");
        reconcile_opt!(&self.model, "model");
        reconcile_opt!(self.resolution, "resolution");
        reconcile_opt!(self.width, "width");
        reconcile_opt!(self.height, "height");
        reconcile_opt!(self.duration, "duration");
        reconcile_opt!(self.fps, "fps");

        Ok(())
    }
}

/// Sparse Hydrate implementation: treats missing keys as None (instead of erroring).
/// This is the counterpart to the sparse Reconcile above.
impl Hydrate for GenerationSettings {
    fn hydrate_map<D: ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
    ) -> Result<Self, HydrateError> {
        // Helper: hydrate Option<T> treating missing keys as None
        fn hydrate_opt_i64<D: ReadDoc>(
            doc: &D,
            obj: &automerge::ObjId,
            key: &str,
        ) -> Result<Option<i64>, HydrateError> {
            match doc.get(obj, key)? {
                None => Ok(None),
                Some((Value::Scalar(s), _)) => match s.as_ref() {
                    ScalarValue::Int(i) => Ok(Some(*i)),
                    ScalarValue::Uint(u) => Ok(Some(*u as i64)),
                    ScalarValue::Null => Ok(None),
                    _ => Ok(None),
                },
                _ => Ok(None),
            }
        }

        fn hydrate_opt_f64<D: ReadDoc>(
            doc: &D,
            obj: &automerge::ObjId,
            key: &str,
        ) -> Result<Option<f64>, HydrateError> {
            match doc.get(obj, key)? {
                None => Ok(None),
                Some((Value::Scalar(s), _)) => match s.as_ref() {
                    ScalarValue::F64(f) => Ok(Some(*f)),
                    ScalarValue::Int(i) => Ok(Some(*i as f64)),
                    ScalarValue::Null => Ok(None),
                    _ => Ok(None),
                },
                _ => Ok(None),
            }
        }

        fn hydrate_opt_i32<D: ReadDoc>(
            doc: &D,
            obj: &automerge::ObjId,
            key: &str,
        ) -> Result<Option<i32>, HydrateError> {
            hydrate_opt_i64(doc, obj, key).map(|opt| opt.map(|v| v as i32))
        }

        fn hydrate_opt_string<D: ReadDoc>(
            doc: &D,
            obj: &automerge::ObjId,
            key: &str,
        ) -> Result<Option<String>, HydrateError> {
            match doc.get(obj, key)? {
                None => Ok(None),
                Some((Value::Scalar(s), _)) => match s.as_ref() {
                    ScalarValue::Str(st) => Ok(Some(st.to_string())),
                    ScalarValue::Null => Ok(None),
                    _ => Ok(None),
                },
                _ => Ok(None),
            }
        }

        Ok(GenerationSettings {
            seed: hydrate_opt_i64(doc, obj, "seed")?,
            cfg: hydrate_opt_f64(doc, obj, "cfg")?,
            num_steps: hydrate_opt_i32(doc, obj, "num_steps")?,
            model: hydrate_opt_string(doc, obj, "model")?,
            resolution: hydrate_opt_i32(doc, obj, "resolution")?,
            width: hydrate_opt_i32(doc, obj, "width")?,
            height: hydrate_opt_i32(doc, obj, "height")?,
            duration: hydrate_opt_i32(doc, obj, "duration")?,
            fps: hydrate_opt_i32(doc, obj, "fps")?,
        })
    }
}

// =============================================================================
// OUTPUT ASSET
// =============================================================================

/// A generated output asset (image/video).
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct OutputAsset {
    /// The URL of the generated asset.
    pub url: String,

    /// The specific seed used for this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Whether this output is selected as the preview.
    #[serde(default)]
    pub is_selected: bool,
}

impl OutputAsset {
    /// Creates a new OutputAsset with just a URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            seed: None,
            is_selected: false,
        }
    }

    /// Builder: Set seed.
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Builder: Set selected flag.
    pub fn with_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_root_default() {
        let root = DocumentRoot::default();
        assert!(root.is_empty());
        assert_eq!(root.len(), 0);
    }

    #[test]
    fn test_generation_node_builder() {
        let settings = GenerationSettings::new()
            .with_seed(42)
            .with_cfg(7.5)
            .with_model("stable-diffusion-xl");

        let node = GenerationNode::new("test-id", "t2i")
            .with_title("My Image")
            .with_prompt("A beautiful sunset")
            .with_negative_prompt("blurry, low quality")
            .with_settings(settings);

        assert_eq!(node.id, "test-id");
        assert_eq!(node.type_, "t2i");
        assert_eq!(node.status, "pending");
        assert_eq!(node.title_str(), "My Image");
        assert_eq!(node.prompt_str(), "A beautiful sunset");
        assert_eq!(node.settings.seed, Some(42));
        assert_eq!(node.settings.cfg, Some(7.5));
    }

    #[test]
    fn test_output_asset_builder() {
        let output = OutputAsset::new("https://example.com/image.png")
            .with_seed(12345)
            .with_selected(true);

        assert_eq!(output.url, "https://example.com/image.png");
        assert_eq!(output.seed, Some(12345));
        assert!(output.is_selected);
    }

    #[test]
    fn test_node_to_json() {
        let node = GenerationNode::new("test-id", "t2i")
            .with_prompt("A test prompt");

        let json = node.to_json_value();
        assert_eq!(json["id"], "test-id");
        assert_eq!(json["prompt"], "A test prompt");
    }
}
