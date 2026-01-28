//! Data models for the collaborative storyboard manager.
//!
//! These structs map to the TypeScript types in `storyboard.ts`.
//! Using autosurgeon derives for automatic CRDT serialization.

use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// DOCUMENT ROOT
// =============================================================================

/// Root document structure for a collaborative storyboard.
/// Maps to TypeScript `Storyboard` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct StoryboardRoot {
    /// Unique identifier
    pub id: String,
    /// Storyboard title
    pub title: String,
    /// Storyboard description
    pub description: String,
    /// Raw script content
    pub script_content: String,

    /// Script file IDs (Files API - for agentic proxy)
    pub script_files: Vec<String>,
    /// Drive file IDs (Drive API - for storage)
    pub drive_file_ids: Vec<String>,

    /// Status: 'draft' | 'processing' | 'ready'
    pub status: String,
    /// Current processing stage: 'extraction' | 'visual_dev' | 'scene_breakdown' | 'completed'
    pub current_stage: String,

    /// Timestamps (milliseconds since epoch)
    pub created_at: i64,
    pub last_updated: i64,

    /// Total shot budget for the storyboard
    pub num_shots: Option<i32>,

    /// Thumbnail image URL
    pub thumbnail_image: Option<String>,

    /// Sync tracking
    pub last_synced_sha: Option<String>,

    /// Encryption tracking - email used to encrypt the data
    pub encrypted_by_email: Option<String>,

    /// Processing stages (characters, props, sets)
    pub processing_stages: ProcessingStages,

    /// Scene ordering (scene IDs)
    pub scene_order: Vec<String>,

    /// Scene data keyed by scene ID
    pub scenes: HashMap<String, Scene>,

    /// Uploaded assets keyed by asset ID
    pub uploaded_assets: HashMap<String, UploadedAsset>,

    /// Metadata
    pub metadata: StoryboardMetadata,
}

impl StoryboardRoot {
    /// Creates a new empty storyboard root with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "draft".to_string(),
            current_stage: "extraction".to_string(),
            ..Default::default()
        }
    }

    /// Builder: Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Set script content.
    pub fn with_script_content(mut self, content: impl Into<String>) -> Self {
        self.script_content = content.into();
        self
    }
}

// =============================================================================
// METADATA
// =============================================================================

/// Storyboard metadata.
/// Maps to TypeScript `StoryMetadata` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct StoryboardMetadata {
    pub num_shots: Option<i32>,
    pub aspect_ratio: Option<String>,
}

// =============================================================================
// PROCESSING STAGES
// =============================================================================

/// Processing stages container for characters, props, and sets.
/// Maps to TypeScript `ProcessingStages` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct ProcessingStages {
    /// Character entities keyed by ID
    pub characters: HashMap<String, Character>,
    /// Character ordering (IDs)
    pub character_order: Vec<String>,

    /// Prop entities keyed by ID
    pub props: HashMap<String, Prop>,
    /// Prop ordering (IDs)
    pub prop_order: Vec<String>,

    /// Set/Location entities keyed by ID
    pub sets: HashMap<String, SetLocation>,
    /// Set ordering (IDs)
    pub set_order: Vec<String>,
}

// =============================================================================
// CHARACTER
// =============================================================================

/// Character entity with generation state.
/// Maps to TypeScript `Character` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_prompt: String,
    pub attributes: HashMap<String, String>,

    /// Tag reference (e.g., "@richie")
    pub tag: Option<String>,
    /// One-line visual description
    pub caption: Option<String>,
    /// Image URL
    pub image: Option<String>,
    /// Whether image was enhanced/uploaded and processed
    pub enhanced: Option<bool>,
    /// ID of the generation that created the image
    pub generation_id: Option<String>,
    /// Current generation status: 'idle' | 'pending' | 'success' | 'failed'
    pub generation_status: Option<String>,
    /// Description generation status: 'idle' | 'pending' | 'generating' | 'success' | 'failed'
    pub description_status: Option<String>,
    /// Error message if description generation failed
    pub description_error: Option<String>,
    /// LoRA model ID
    pub lora_model_id: Option<String>,
    /// History of previous images (max 20)
    pub history: Vec<AssetHistory>,
}

impl Character {
    /// Creates a new Character with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder: Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Set tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Builder: Set image prompt.
    pub fn with_image_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.image_prompt = prompt.into();
        self
    }
}

// =============================================================================
// PROP
// =============================================================================

/// Prop entity with generation state.
/// Maps to TypeScript `Prop` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Prop {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_prompt: String,

    pub tag: Option<String>,
    pub caption: Option<String>,
    pub image: Option<String>,
    /// Original image before enhancement (for revert)
    pub original_image: Option<String>,
    pub enhanced: Option<bool>,
    pub generation_id: Option<String>,
    pub generation_status: Option<String>,
    pub description_status: Option<String>,
    pub description_error: Option<String>,
    pub lora_model_id: Option<String>,
    pub history: Vec<AssetHistory>,
}

impl Prop {
    /// Creates a new Prop with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder: Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Set tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
}

// =============================================================================
// SET LOCATION
// =============================================================================

/// Set/Location entity with generation state.
/// Maps to TypeScript `SetLocation` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SetLocation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_prompt: String,

    pub tag: Option<String>,
    pub caption: Option<String>,
    pub image: Option<String>,
    pub enhanced: Option<bool>,
    pub generation_id: Option<String>,
    pub generation_status: Option<String>,
    pub description_status: Option<String>,
    pub description_error: Option<String>,
    pub lora_model_id: Option<String>,
    pub history: Vec<AssetHistory>,
}

impl SetLocation {
    /// Creates a new SetLocation with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder: Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Set tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
}

// =============================================================================
// SCENE
// =============================================================================

/// Scene with shots and per-character looks/outfits.
/// Maps to TypeScript `Scene` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Scene {
    pub id: String,
    pub scene_number: i32,
    pub title: String,
    /// Scene header (e.g., "INT. OFFICE - DAY")
    pub header: String,
    /// Raw script text
    pub content: String,

    /// Visual density score (1-10, deprecated - use predicted_shots)
    pub visual_density_score: i32,
    /// LLM-predicted number of shots for this scene
    pub predicted_shots: i32,
    /// Reasoning for shot count
    pub reasoning: String,

    /// Characters present (IDs) - Phase 1 backward compat
    pub characters_present: Vec<String>,

    /// Set reference (ID) - Phase 1 backward compat
    pub set_ref: Option<String>,
    /// Scene synopsis
    pub synopsis: Option<String>,
    /// Time of day from header
    pub time: Option<String>,
    /// Raw scene text (alias for content)
    pub raw_text: Option<String>,
    /// Overall looks description
    pub looks_description: Option<String>,
    /// Overall outfit description
    pub outfit_description: Option<String>,

    /// Entity references using TAGS
    pub known_entities: Option<KnownEntities>,

    /// Per-character looks keyed by TAG (e.g., "@richie")
    pub character_looks: HashMap<String, CharacterLook>,
    /// Per-character outfits keyed by TAG
    pub character_outfits: HashMap<String, CharacterOutfit>,
    /// Combined looks + outfit images keyed by TAG
    pub looks_with_outfit: HashMap<String, LooksWithOutfit>,

    /// Legacy outfits map (backward compat)
    pub outfits: HashMap<String, OutfitEntry>,

    /// Shot ordering (shot IDs)
    pub shot_order: Vec<String>,
    /// Shot data keyed by shot ID
    pub shots: HashMap<String, Shot>,
}

impl Scene {
    /// Creates a new Scene with the given ID and scene number.
    pub fn new(id: impl Into<String>, scene_number: i32) -> Self {
        Self {
            id: id.into(),
            scene_number,
            ..Default::default()
        }
    }

    /// Builder: Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: Set header.
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = header.into();
        self
    }

    /// Builder: Set content.
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
}

/// Entity references for a scene.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct KnownEntities {
    pub characters: Vec<EntityRef>,
    pub sets: Vec<EntityRef>,
    pub props: Vec<EntityRef>,
}

/// Entity reference with tag and name.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct EntityRef {
    pub tag: String,
    pub name: String,
}

/// Character look for a specific scene.
/// Maps to TypeScript `CharacterLook` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CharacterLook {
    /// Physical appearance: face, body, movement, intensity
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
    pub caption: Option<String>,
    pub enhanced: Option<bool>,
    pub history: Vec<AssetHistory>,
}

/// Character outfit for a specific scene.
/// Maps to TypeScript `CharacterOutfit` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CharacterOutfit {
    /// Garments, colors, materials, style, accessories
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
    pub caption: Option<String>,
    pub history: Vec<AssetHistory>,
}

/// Combined looks + outfit image.
/// Maps to TypeScript `LooksWithOutfit` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LooksWithOutfit {
    pub image: Option<String>,
    pub generation_id: Option<String>,
    pub prompt: Option<String>,
    pub caption: Option<String>,
}

/// Legacy outfit entry (backward compat).
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct OutfitEntry {
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
}

// =============================================================================
// SHOT
// =============================================================================

/// Shot with visual continuity references.
/// Maps to TypeScript `Shot` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Shot {
    pub id: String,
    pub shot_number: i32,
    pub image_prompt: String,

    /// Phase 1 fields (backward compat)
    pub size: String,
    pub angle: String,
    pub visual_description: String,
    pub assets_used: Vec<String>,

    /// Image URL
    pub image: Option<String>,
    /// Current generation status
    pub generation_status: Option<String>,

    /// Phase 2 fields
    pub assets: Option<Vec<AssetRef>>,
    pub environment: Option<String>,
    pub action: Option<String>,
    pub camera: Option<String>,
    pub additional_instructions: Option<String>,
    pub known_assets: Option<ShotKnownAssets>,

    /// Deprecated fields
    pub title: Option<String>,
    pub visual_prompt: Option<String>,
    pub camera_type: Option<String>,
    pub camera_angle: Option<String>,

    /// Phase 3: Visual Continuity
    /// Subject being focused in this shot (entity tag)
    pub subject: Option<String>,
    /// Reference shot for visual continuity
    /// -1 = First establishing shot or completely new angle/visual
    /// N = Reference to shot number N (N must be < current shot_number)
    pub ref_shot_id: Option<i32>,

    /// History for undo (max 20 items)
    pub history: Vec<ShotHistory>,
}

impl Shot {
    /// Creates a new Shot with the given ID and shot number.
    pub fn new(id: impl Into<String>, shot_number: i32) -> Self {
        Self {
            id: id.into(),
            shot_number,
            ..Default::default()
        }
    }

    /// Builder: Set image prompt.
    pub fn with_image_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.image_prompt = prompt.into();
        self
    }

    /// Builder: Set action.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Builder: Set camera.
    pub fn with_camera(mut self, camera: impl Into<String>) -> Self {
        self.camera = Some(camera.into());
        self
    }
}

/// Asset reference with tag and name.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct AssetRef {
    pub tag: String,
    pub name: String,
}

/// Known assets for a shot.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct ShotKnownAssets {
    /// Keyed by character TAG (e.g., "@richie")
    pub characters: HashMap<String, ShotCharacterRef>,
    pub sets: Vec<ShotAssetRef>,
    pub props: Vec<ShotAssetRef>,
}

/// Character reference for a shot.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct ShotCharacterRef {
    /// Physical appearance (NOT outfit)
    pub description: String,
    /// Outfit description
    pub outfit: String,
    /// Reference images (resolved by UI) - Priority order
    pub looks_with_outfit_image: Option<String>,
    pub looks_image: Option<String>,
    pub outfit_image: Option<String>,
    pub character_image: Option<String>,
}

/// Asset reference for a shot (sets/props).
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct ShotAssetRef {
    pub tag: String,
    pub name: String,
    pub image: Option<String>,
}

// =============================================================================
// HISTORY
// =============================================================================

/// Shot history entry.
/// Maps to TypeScript `ShotHistory` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ShotHistory {
    pub id: String,
    pub image: String,
    pub prompt: String,
    pub timestamp: i64,
}

impl ShotHistory {
    /// Creates a new ShotHistory entry.
    pub fn new(id: impl Into<String>, image: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            image: image.into(),
            prompt: prompt.into(),
            timestamp: 0,
        }
    }

    /// Builder: Set timestamp.
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Asset history entry.
/// Maps to TypeScript `AssetHistory` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AssetHistory {
    pub id: String,
    pub image: String,
    pub image_prompt: String,
    pub generation_id: Option<String>,
    pub lora_model_id: Option<String>,
    pub timestamp: i64,
}

impl AssetHistory {
    /// Creates a new AssetHistory entry.
    pub fn new(id: impl Into<String>, image: impl Into<String>, image_prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            image: image.into(),
            image_prompt: image_prompt.into(),
            timestamp: 0,
            ..Default::default()
        }
    }

    /// Builder: Set timestamp.
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Builder: Set generation ID.
    pub fn with_generation_id(mut self, generation_id: impl Into<String>) -> Self {
        self.generation_id = Some(generation_id.into());
        self
    }
}

// =============================================================================
// UPLOADED ASSET
// =============================================================================

/// Uploaded asset from local system.
/// Maps to TypeScript `UploadedAsset` interface.
#[derive(Debug, Clone, Default, Reconcile, Hydrate, Serialize, Deserialize, PartialEq)]
pub struct UploadedAsset {
    pub id: String,
    pub name: String,
    /// Data URL or URL
    pub image: String,
    /// MIME type
    pub file_type: String,
    /// Size in bytes
    pub file_size: i64,
    /// Timestamp when uploaded
    pub uploaded_at: i64,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storyboard_root_default() {
        let root = StoryboardRoot::default();
        assert!(root.id.is_empty());
        assert!(root.scenes.is_empty());
        assert!(root.processing_stages.characters.is_empty());
    }

    #[test]
    fn test_storyboard_root_builder() {
        let root = StoryboardRoot::new("test-id")
            .with_title("My Storyboard")
            .with_description("A test storyboard");

        assert_eq!(root.id, "test-id");
        assert_eq!(root.title, "My Storyboard");
        assert_eq!(root.status, "draft");
        assert_eq!(root.current_stage, "extraction");
    }

    #[test]
    fn test_character_builder() {
        let character = Character::new("char-1", "John")
            .with_description("A tall man with dark hair")
            .with_tag("@john");

        assert_eq!(character.id, "char-1");
        assert_eq!(character.name, "John");
        assert_eq!(character.tag, Some("@john".to_string()));
    }

    #[test]
    fn test_scene_builder() {
        let scene = Scene::new("scene-1", 1)
            .with_title("Opening Scene")
            .with_header("INT. OFFICE - DAY");

        assert_eq!(scene.id, "scene-1");
        assert_eq!(scene.scene_number, 1);
        assert_eq!(scene.header, "INT. OFFICE - DAY");
    }

    #[test]
    fn test_shot_builder() {
        let shot = Shot::new("shot-1", 1)
            .with_image_prompt("A wide shot of the office")
            .with_action("John enters the room");

        assert_eq!(shot.id, "shot-1");
        assert_eq!(shot.shot_number, 1);
        assert_eq!(shot.action, Some("John enters the room".to_string()));
    }

    #[test]
    fn test_history_builder() {
        let history = AssetHistory::new("h-1", "https://example.com/img.png", "A test prompt")
            .with_timestamp(1234567890)
            .with_generation_id("gen-123");

        assert_eq!(history.id, "h-1");
        assert_eq!(history.timestamp, 1234567890);
        assert_eq!(history.generation_id, Some("gen-123".to_string()));
    }
}
