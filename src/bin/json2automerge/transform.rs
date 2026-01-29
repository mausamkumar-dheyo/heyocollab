//! Transformation logic from input JSON structs to Rust model structs.
//!
//! Key transformations:
//! - Arrays → HashMap + order Vec
//! - Nested shots array in Scene → HashMap + shot_order

use std::collections::HashMap;

use crate::input::*;
use heyocollab::storyboard::model::*;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Converts an array to HashMap + order vector, preserving original order.
fn array_to_hashmap<T, F>(items: Vec<T>, key_fn: F) -> (HashMap<String, T>, Vec<String>)
where
    F: Fn(&T) -> String,
{
    let order: Vec<String> = items.iter().map(&key_fn).collect();
    let map: HashMap<String, T> = items.into_iter().map(|item| (key_fn(&item), item)).collect();
    (map, order)
}

// =============================================================================
// ROOT STORYBOARD
// =============================================================================

impl From<InputStoryboard> for StoryboardRoot {
    fn from(input: InputStoryboard) -> Self {
        // Transform processing_stages arrays
        let (characters, character_order) =
            array_to_hashmap(input.data.processing_stages.characters, |c| c.id.clone());
        let characters: HashMap<String, Character> =
            characters.into_iter().map(|(k, v)| (k, v.into())).collect();

        let (props, prop_order) =
            array_to_hashmap(input.data.processing_stages.props, |p| p.id.clone());
        let props: HashMap<String, Prop> =
            props.into_iter().map(|(k, v)| (k, v.into())).collect();

        let (sets, set_order) =
            array_to_hashmap(input.data.processing_stages.sets, |s| s.id.clone());
        let sets: HashMap<String, SetLocation> =
            sets.into_iter().map(|(k, v)| (k, v.into())).collect();

        // Transform scenes array (also transforms nested shots)
        let (scenes, scene_order) = transform_scenes(input.data.scenes);

        // Transform uploaded_assets
        let (uploaded_assets, _) =
            array_to_hashmap(input.data.uploaded_assets, |a| a.id.clone());
        let uploaded_assets: HashMap<String, UploadedAsset> = uploaded_assets
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        StoryboardRoot {
            id: input.id,
            title: input.title,
            description: input.description,
            script_content: input.script_content,
            script_files: input.script_files,
            drive_file_ids: input.drive_file_ids,
            status: input.status,
            current_stage: input.current_stage,
            created_at: input.created_at,
            last_updated: input.last_updated,
            num_shots: input.num_shots,
            thumbnail_image: input.thumbnail_image,
            last_synced_sha: input.last_synced_sha,
            encrypted_by_email: input.encrypted_by_email,

            processing_stages: ProcessingStages {
                characters,
                character_order,
                props,
                prop_order,
                sets,
                set_order,
            },

            scene_order,
            scenes,
            uploaded_assets,
            metadata: input.data.metadata.map(|m| m.into()).unwrap_or_default(),
        }
    }
}

/// Transform scenes array, also converting shots arrays within each scene.
fn transform_scenes(input_scenes: Vec<InputScene>) -> (HashMap<String, Scene>, Vec<String>) {
    let scene_order: Vec<String> = input_scenes.iter().map(|s| s.id.clone()).collect();

    let scenes: HashMap<String, Scene> = input_scenes
        .into_iter()
        .map(|input_scene| {
            let id = input_scene.id.clone();
            (id, input_scene.into())
        })
        .collect();

    (scenes, scene_order)
}

// =============================================================================
// METADATA
// =============================================================================

impl From<InputMetadata> for StoryboardMetadata {
    fn from(input: InputMetadata) -> Self {
        Self {
            num_shots: input.num_shots,
            aspect_ratio: input.aspect_ratio,
        }
    }
}

// =============================================================================
// ENTITIES
// =============================================================================

impl From<InputCharacter> for Character {
    fn from(input: InputCharacter) -> Self {
        Self {
            id: input.id,
            name: input.name,
            description: input.description,
            image_prompt: input.image_prompt,
            attributes: input.attributes,
            tag: input.tag,
            caption: input.caption,
            image: input.image,
            enhanced: input.enhanced,
            generation_id: input.generation_id,
            generation_status: input.generation_status,
            description_status: input.description_status,
            description_error: input.description_error,
            lora_model_id: input.lora_model_id,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

impl From<InputProp> for Prop {
    fn from(input: InputProp) -> Self {
        Self {
            id: input.id,
            name: input.name,
            description: input.description,
            image_prompt: input.image_prompt,
            tag: input.tag,
            caption: input.caption,
            image: input.image,
            original_image: input.original_image,
            enhanced: input.enhanced,
            generation_id: input.generation_id,
            generation_status: input.generation_status,
            description_status: input.description_status,
            description_error: input.description_error,
            lora_model_id: input.lora_model_id,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

impl From<InputSetLocation> for SetLocation {
    fn from(input: InputSetLocation) -> Self {
        Self {
            id: input.id,
            name: input.name,
            description: input.description,
            image_prompt: input.image_prompt,
            tag: input.tag,
            caption: input.caption,
            image: input.image,
            enhanced: input.enhanced,
            generation_id: input.generation_id,
            generation_status: input.generation_status,
            description_status: input.description_status,
            description_error: input.description_error,
            lora_model_id: input.lora_model_id,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

// =============================================================================
// SCENE
// =============================================================================

impl From<InputScene> for Scene {
    fn from(input: InputScene) -> Self {
        // Transform shots array to HashMap + order
        let shot_order: Vec<String> = input.shots.iter().map(|s| s.id.clone()).collect();
        let shots: HashMap<String, Shot> = input
            .shots
            .into_iter()
            .map(|s| (s.id.clone(), s.into()))
            .collect();

        Self {
            id: input.id,
            scene_number: input.scene_number,
            title: input.title,
            header: input.header,
            content: input.content,
            visual_density_score: input.visual_density_score,
            predicted_shots: input.predicted_shots,
            reasoning: input.reasoning,
            characters_present: input.characters_present,
            set_ref: input.set_ref,
            synopsis: input.synopsis,
            time: input.time,
            raw_text: input.raw_text,
            looks_description: input.looks_description,
            outfit_description: input.outfit_description,
            known_entities: input.known_entities.map(|ke| ke.into()),
            character_looks: input
                .character_looks
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            character_outfits: input
                .character_outfits
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            looks_with_outfit: input
                .looks_with_outfit
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            outfits: input
                .outfits
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            shot_order,
            shots,
        }
    }
}

impl From<InputKnownEntities> for KnownEntities {
    fn from(input: InputKnownEntities) -> Self {
        Self {
            characters: input.characters.into_iter().map(|e| e.into()).collect(),
            sets: input.sets.into_iter().map(|e| e.into()).collect(),
            props: input.props.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl From<InputEntityRef> for EntityRef {
    fn from(input: InputEntityRef) -> Self {
        Self {
            tag: input.tag,
            name: input.name,
        }
    }
}

impl From<InputCharacterLook> for CharacterLook {
    fn from(input: InputCharacterLook) -> Self {
        Self {
            description: input.description,
            image: input.image,
            image_prompt: input.image_prompt,
            generation_id: input.generation_id,
            caption: input.caption,
            enhanced: input.enhanced,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

impl From<InputCharacterOutfit> for CharacterOutfit {
    fn from(input: InputCharacterOutfit) -> Self {
        Self {
            description: input.description,
            image: input.image,
            image_prompt: input.image_prompt,
            generation_id: input.generation_id,
            caption: input.caption,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

impl From<InputLooksWithOutfit> for LooksWithOutfit {
    fn from(input: InputLooksWithOutfit) -> Self {
        Self {
            image: input.image,
            generation_id: input.generation_id,
            prompt: input.prompt,
            caption: input.caption,
        }
    }
}

impl From<InputOutfitEntry> for OutfitEntry {
    fn from(input: InputOutfitEntry) -> Self {
        Self {
            description: input.description,
            image: input.image,
            image_prompt: input.image_prompt,
            generation_id: input.generation_id,
        }
    }
}

// =============================================================================
// SHOT
// =============================================================================

impl From<InputShot> for Shot {
    fn from(input: InputShot) -> Self {
        Self {
            id: input.id,
            shot_number: input.shot_number,
            image_prompt: input.image_prompt,
            size: input.size,
            angle: input.angle,
            visual_description: input.visual_description,
            assets_used: input.assets_used,
            image: input.image,
            generation_status: input.generation_status,
            assets: input.assets.map(|v| v.into_iter().map(|a| a.into()).collect()),
            environment: input.environment,
            action: input.action,
            camera: input.camera,
            additional_instructions: input.additional_instructions,
            known_assets: input.known_assets.map(|ka| ka.into()),
            title: input.title,
            visual_prompt: input.visual_prompt,
            camera_type: input.camera_type,
            camera_angle: input.camera_angle,
            subject: input.subject,
            ref_shot_id: input.ref_shot_id,
            history: input.history.into_iter().map(|h| h.into()).collect(),
        }
    }
}

impl From<InputAssetRef> for AssetRef {
    fn from(input: InputAssetRef) -> Self {
        Self {
            tag: input.tag,
            name: input.name,
        }
    }
}

impl From<InputShotKnownAssets> for ShotKnownAssets {
    fn from(input: InputShotKnownAssets) -> Self {
        Self {
            characters: input
                .characters
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            sets: input.sets.into_iter().map(|a| a.into()).collect(),
            props: input.props.into_iter().map(|a| a.into()).collect(),
        }
    }
}

impl From<InputShotCharacterRef> for ShotCharacterRef {
    fn from(input: InputShotCharacterRef) -> Self {
        Self {
            description: input.description,
            outfit: input.outfit,
            looks_with_outfit_image: input.looks_with_outfit_image,
            looks_image: input.looks_image,
            outfit_image: input.outfit_image,
            character_image: input.character_image,
        }
    }
}

impl From<InputShotAssetRef> for ShotAssetRef {
    fn from(input: InputShotAssetRef) -> Self {
        Self {
            tag: input.tag,
            name: input.name,
            image: input.image,
        }
    }
}

// =============================================================================
// HISTORY
// =============================================================================

impl From<InputAssetHistory> for AssetHistory {
    fn from(input: InputAssetHistory) -> Self {
        Self {
            id: input.id,
            image: input.image,
            image_prompt: input.image_prompt,
            generation_id: input.generation_id,
            lora_model_id: input.lora_model_id,
            timestamp: input.timestamp,
        }
    }
}

impl From<InputShotHistory> for ShotHistory {
    fn from(input: InputShotHistory) -> Self {
        Self {
            id: input.id,
            image: input.image,
            prompt: input.prompt,
            timestamp: input.timestamp,
        }
    }
}

// =============================================================================
// UPLOADED ASSET
// =============================================================================

impl From<InputUploadedAsset> for UploadedAsset {
    fn from(input: InputUploadedAsset) -> Self {
        Self {
            id: input.id,
            name: input.name,
            image: input.image,
            file_type: input.file_type,
            file_size: input.file_size,
            uploaded_at: input.uploaded_at,
        }
    }
}
