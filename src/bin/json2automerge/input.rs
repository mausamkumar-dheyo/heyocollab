//! Input structs for parsing TypeScript storyboard JSON.
//!
//! These structs match the TypeScript interfaces in storyboard.ts.
//! Key differences from Rust model:
//! - Uses arrays instead of HashMaps (transformed later)
//! - Uses camelCase for some root-level fields

use serde::Deserialize;
use std::collections::HashMap;

// =============================================================================
// ROOT STORYBOARD
// =============================================================================

/// Root storyboard structure from TypeScript.
/// Some fields are camelCase (scriptContent, createdAt, etc.)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputStoryboard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub script_content: String,

    #[serde(default)]
    pub script_files: Vec<String>,
    #[serde(default)]
    pub drive_file_ids: Vec<String>,

    pub thumbnail_image: Option<String>,

    pub created_at: i64,
    pub last_updated: i64,

    pub num_shots: Option<i32>,

    pub status: String,
    pub current_stage: String,

    pub last_synced_sha: Option<String>,
    pub encrypted_by_email: Option<String>,

    pub data: InputStoryData,
}

/// Story data container.
#[derive(Debug, Deserialize)]
pub struct InputStoryData {
    pub processing_stages: InputProcessingStages,
    pub scenes: Vec<InputScene>,
    pub metadata: Option<InputMetadata>,
    #[serde(default)]
    #[serde(rename = "uploadedAssets")]
    pub uploaded_assets: Vec<InputUploadedAsset>,
}

/// Processing stages with arrays.
#[derive(Debug, Deserialize)]
pub struct InputProcessingStages {
    pub characters: Vec<InputCharacter>,
    pub props: Vec<InputProp>,
    pub sets: Vec<InputSetLocation>,
}

/// Story metadata.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMetadata {
    pub num_shots: Option<i32>,
    pub aspect_ratio: Option<String>,
}

// =============================================================================
// ENTITIES (Character, Prop, SetLocation)
// =============================================================================

/// Character entity.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InputCharacter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_prompt: String,
    pub attributes: HashMap<String, String>,

    pub tag: Option<String>,
    pub caption: Option<String>,
    pub image: Option<String>,
    pub enhanced: Option<bool>,
    pub generation_id: Option<String>,
    pub generation_status: Option<String>,
    pub description_status: Option<String>,
    pub description_error: Option<String>,
    #[serde(rename = "loraModelId")]
    pub lora_model_id: Option<String>,
    pub history: Vec<InputAssetHistory>,
}

impl Default for InputCharacter {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            image_prompt: String::new(),
            attributes: HashMap::new(),
            tag: None,
            caption: None,
            image: None,
            enhanced: None,
            generation_id: None,
            generation_status: None,
            description_status: None,
            description_error: None,
            lora_model_id: None,
            history: Vec::new(),
        }
    }
}

/// Prop entity.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InputProp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_prompt: String,

    pub tag: Option<String>,
    pub caption: Option<String>,
    pub image: Option<String>,
    pub original_image: Option<String>,
    pub enhanced: Option<bool>,
    pub generation_id: Option<String>,
    pub generation_status: Option<String>,
    pub description_status: Option<String>,
    pub description_error: Option<String>,
    #[serde(rename = "loraModelId")]
    pub lora_model_id: Option<String>,
    pub history: Vec<InputAssetHistory>,
}

impl Default for InputProp {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            image_prompt: String::new(),
            tag: None,
            caption: None,
            image: None,
            original_image: None,
            enhanced: None,
            generation_id: None,
            generation_status: None,
            description_status: None,
            description_error: None,
            lora_model_id: None,
            history: Vec::new(),
        }
    }
}

/// Set/Location entity.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InputSetLocation {
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
    #[serde(rename = "loraModelId")]
    pub lora_model_id: Option<String>,
    pub history: Vec<InputAssetHistory>,
}

impl Default for InputSetLocation {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            image_prompt: String::new(),
            tag: None,
            caption: None,
            image: None,
            enhanced: None,
            generation_id: None,
            generation_status: None,
            description_status: None,
            description_error: None,
            lora_model_id: None,
            history: Vec::new(),
        }
    }
}

// =============================================================================
// SCENE
// =============================================================================

/// Scene with shots array.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InputScene {
    pub id: String,
    pub scene_number: i32,
    pub title: String,
    pub header: String,
    pub content: String,

    pub visual_density_score: i32,
    pub predicted_shots: i32,
    pub reasoning: String,

    pub characters_present: Vec<String>,
    pub set_ref: Option<String>,
    pub synopsis: Option<String>,
    pub time: Option<String>,
    pub raw_text: Option<String>,
    pub looks_description: Option<String>,
    pub outfit_description: Option<String>,

    pub known_entities: Option<InputKnownEntities>,
    pub character_looks: HashMap<String, InputCharacterLook>,
    pub character_outfits: HashMap<String, InputCharacterOutfit>,
    pub looks_with_outfit: HashMap<String, InputLooksWithOutfit>,
    pub outfits: HashMap<String, InputOutfitEntry>,

    pub shots: Vec<InputShot>,
}

impl Default for InputScene {
    fn default() -> Self {
        Self {
            id: String::new(),
            scene_number: 0,
            title: String::new(),
            header: String::new(),
            content: String::new(),
            visual_density_score: 0,
            predicted_shots: 0,
            reasoning: String::new(),
            characters_present: Vec::new(),
            set_ref: None,
            synopsis: None,
            time: None,
            raw_text: None,
            looks_description: None,
            outfit_description: None,
            known_entities: None,
            character_looks: HashMap::new(),
            character_outfits: HashMap::new(),
            looks_with_outfit: HashMap::new(),
            outfits: HashMap::new(),
            shots: Vec::new(),
        }
    }
}

/// Entity references for a scene.
#[derive(Debug, Deserialize, Default)]
pub struct InputKnownEntities {
    #[serde(default)]
    pub characters: Vec<InputEntityRef>,
    #[serde(default)]
    pub sets: Vec<InputEntityRef>,
    #[serde(default)]
    pub props: Vec<InputEntityRef>,
}

/// Entity reference.
#[derive(Debug, Deserialize, Default)]
pub struct InputEntityRef {
    pub tag: String,
    pub name: String,
}

/// Character look for a scene.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputCharacterLook {
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
    pub caption: Option<String>,
    pub enhanced: Option<bool>,
    pub history: Vec<InputAssetHistory>,
}

/// Character outfit for a scene.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputCharacterOutfit {
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
    pub caption: Option<String>,
    pub history: Vec<InputAssetHistory>,
}

/// Combined looks + outfit.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputLooksWithOutfit {
    pub image: Option<String>,
    pub generation_id: Option<String>,
    pub prompt: Option<String>,
    pub caption: Option<String>,
}

/// Legacy outfit entry.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputOutfitEntry {
    pub description: String,
    pub image: Option<String>,
    pub image_prompt: Option<String>,
    pub generation_id: Option<String>,
}

// =============================================================================
// SHOT
// =============================================================================

/// Shot with all fields.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InputShot {
    pub id: String,
    pub shot_number: i32,
    pub image_prompt: String,

    // Phase 1 fields
    pub size: String,
    pub angle: String,
    pub visual_description: String,
    pub assets_used: Vec<String>,

    pub image: Option<String>,
    pub generation_status: Option<String>,

    // Phase 2 fields
    pub assets: Option<Vec<InputAssetRef>>,
    pub environment: Option<String>,
    pub action: Option<String>,
    pub camera: Option<String>,
    pub additional_instructions: Option<String>,
    pub known_assets: Option<InputShotKnownAssets>,

    // Deprecated fields
    pub title: Option<String>,
    pub visual_prompt: Option<String>,
    pub camera_type: Option<String>,
    pub camera_angle: Option<String>,

    // Phase 3 fields
    pub subject: Option<String>,
    pub ref_shot_id: Option<i32>,

    pub history: Vec<InputShotHistory>,
}

impl Default for InputShot {
    fn default() -> Self {
        Self {
            id: String::new(),
            shot_number: 0,
            image_prompt: String::new(),
            size: String::new(),
            angle: String::new(),
            visual_description: String::new(),
            assets_used: Vec::new(),
            image: None,
            generation_status: None,
            assets: None,
            environment: None,
            action: None,
            camera: None,
            additional_instructions: None,
            known_assets: None,
            title: None,
            visual_prompt: None,
            camera_type: None,
            camera_angle: None,
            subject: None,
            ref_shot_id: None,
            history: Vec::new(),
        }
    }
}

/// Asset reference.
#[derive(Debug, Deserialize, Default)]
pub struct InputAssetRef {
    pub tag: String,
    pub name: String,
}

/// Known assets for a shot.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputShotKnownAssets {
    pub characters: HashMap<String, InputShotCharacterRef>,
    pub sets: Vec<InputShotAssetRef>,
    pub props: Vec<InputShotAssetRef>,
}

/// Character reference in a shot.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputShotCharacterRef {
    pub description: String,
    pub outfit: String,
    pub looks_with_outfit_image: Option<String>,
    pub looks_image: Option<String>,
    pub outfit_image: Option<String>,
    pub character_image: Option<String>,
}

/// Asset reference in a shot.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputShotAssetRef {
    pub tag: String,
    pub name: String,
    pub image: Option<String>,
}

// =============================================================================
// HISTORY
// =============================================================================

/// Asset history entry.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputAssetHistory {
    pub id: String,
    pub image: String,
    pub image_prompt: String,
    pub generation_id: Option<String>,
    #[serde(rename = "loraModelId")]
    pub lora_model_id: Option<String>,
    pub timestamp: i64,
}

/// Shot history entry.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct InputShotHistory {
    pub id: String,
    pub image: String,
    pub prompt: String,
    pub timestamp: i64,
}

// =============================================================================
// UPLOADED ASSET
// =============================================================================

/// Uploaded asset.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct InputUploadedAsset {
    pub id: String,
    pub name: String,
    pub image: String,
    pub file_type: String,
    pub file_size: i64,
    pub uploaded_at: i64,
}
