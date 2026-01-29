//! Migration orchestration

use crate::client::{ClientError, HeyoClient};
use crate::compression::maybe_decompress;
use crate::crypto::{decrypt_data, CryptoError, KeyParams};
use heyocollab::storyboard::{StoryboardManager, StoryboardRoot};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

/// Migration errors
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Client error: {0}")]
    Client(#[from] ClientError),
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Automerge error: {0}")]
    Automerge(String),
    #[error("Missing field: {0}")]
    MissingField(String),
}

/// Storyboard file structure (works for both encrypted and plain)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinFile {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub script_content: String,
    pub created_at: i64,
    #[serde(default)]
    pub last_updated: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub current_stage: String,
    #[serde(default)]
    pub script_files: Vec<String>,
    #[serde(default)]
    pub drive_file_ids: Vec<String>,
    #[serde(default)]
    pub num_shots: Option<i32>,
    #[serde(default)]
    pub thumbnail_image: Option<String>,
    #[serde(default)]
    pub last_synced_sha: Option<String>,
    pub encrypted_by_email: Option<String>,
    /// Data field - can be encrypted ({ "_": "..." }) or plain ({ "processing_stages": {...} })
    pub data: Value,
}

/// Result of a single storyboard migration
#[derive(Debug)]
pub struct MigrationResult {
    pub storyboard_id: String,
    pub title: String,
    pub success: bool,
    pub error: Option<String>,
    pub input_size: usize,
    pub output_size: usize,
    pub skipped: bool,
}

impl MigrationResult {
    #[allow(dead_code)]
    fn error(id: &str, title: &str, msg: impl Into<String>) -> Self {
        Self {
            storyboard_id: id.to_string(),
            title: title.to_string(),
            success: false,
            error: Some(msg.into()),
            input_size: 0,
            output_size: 0,
            skipped: false,
        }
    }

    #[allow(dead_code)]
    fn skipped(id: &str, title: &str) -> Self {
        Self {
            storyboard_id: id.to_string(),
            title: title.to_string(),
            success: true,
            error: None,
            input_size: 0,
            output_size: 0,
            skipped: true,
        }
    }
}

/// Check if data field is encrypted (has "_" key with base64 string)
fn is_encrypted(data: &Value) -> bool {
    data.get("_").map(|v| v.is_string()).unwrap_or(false)
}

/// Migrate a single storyboard
pub async fn migrate_storyboard(
    client: &HeyoClient,
    storyboard_id: &str,
    skip_upload: bool,
    output_dir: Option<&Path>,
    _force: bool,
) -> MigrationResult {
    let mut result = MigrationResult {
        storyboard_id: storyboard_id.to_string(),
        title: String::new(),
        success: false,
        error: None,
        input_size: 0,
        output_size: 0,
        skipped: false,
    };

    // 1. Get latest file metadata
    let file_meta = match client.get_latest_sb_file(storyboard_id).await {
        Ok(meta) => meta,
        Err(e) => {
            result.error = Some(format!("Failed to get file metadata: {}", e));
            return result;
        }
    };

    // 2. Download file
    let raw_data = match client.download_file(&file_meta.sb_file_id).await {
        Ok(data) => data,
        Err(e) => {
            result.error = Some(format!("Failed to download file: {}", e));
            return result;
        }
    };
    result.input_size = raw_data.len();

    // 3. Decompress if gzipped
    let decompressed = match maybe_decompress(raw_data) {
        Ok(data) => data,
        Err(e) => {
            result.error = Some(format!("Decompression failed: {}", e));
            return result;
        }
    };

    // 4. Parse JSON structure
    let bin_file: BinFile = match serde_json::from_slice(&decompressed) {
        Ok(f) => f,
        Err(e) => {
            result.error = Some(format!("JSON parse error: {}", e));
            return result;
        }
    };
    result.title = bin_file.title.clone();

    // 5. Get decrypted data (handle both encrypted and plain formats)
    let data_value: Value = if is_encrypted(&bin_file.data) {
        // Encrypted format: { "_": "base64_encrypted_data" }
        let encrypted_str = match bin_file.data.get("_").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                result.error = Some("Invalid encrypted data format".to_string());
                return result;
            }
        };

        // Get encryption email
        let email = match &bin_file.encrypted_by_email {
            Some(e) if !e.is_empty() => e.clone(),
            _ => {
                result.error = Some("Missing encryptedByEmail field for encrypted data".to_string());
                return result;
            }
        };

        // Decrypt
        let key_params = KeyParams {
            email,
            created_at: bin_file.created_at,
        };

        let decrypted_json = match decrypt_data(encrypted_str, &key_params) {
            Ok(json) => json,
            Err(e) => {
                result.error = Some(format!("Decryption failed: {}", e));
                return result;
            }
        };

        // Parse decrypted JSON
        match serde_json::from_str(&decrypted_json) {
            Ok(v) => v,
            Err(e) => {
                result.error = Some(format!("Failed to parse decrypted data: {}", e));
                return result;
            }
        }
    } else {
        // Plain format: data is already the actual data object
        bin_file.data.clone()
    };

    // 6. Reconstruct full storyboard JSON
    let full_json = match reconstruct_storyboard_json(&bin_file, &data_value) {
        Ok(json) => json,
        Err(e) => {
            result.error = Some(format!("Failed to reconstruct JSON: {}", e));
            return result;
        }
    };

    // 7. Parse as InputStoryboard
    let input: crate::input::InputStoryboard = match serde_json::from_str(&full_json) {
        Ok(s) => s,
        Err(e) => {
            result.error = Some(format!("Failed to parse storyboard: {}", e));
            return result;
        }
    };

    // 8. Transform to Automerge
    let root: StoryboardRoot = input.into();

    // 9. Create Automerge document
    let mut manager = StoryboardManager::new();
    if let Err(e) = manager.update_state(|state| *state = root) {
        result.error = Some(format!("Automerge update failed: {}", e));
        return result;
    }

    // 10. Save to binary
    let automerge_binary = manager.save();
    result.output_size = automerge_binary.len();

    // 11. Save locally if output_dir specified
    if let Some(dir) = output_dir {
        let filename = format!("{}.automerge", storyboard_id);
        let path = dir.join(&filename);
        if let Err(e) = std::fs::write(&path, &automerge_binary) {
            result.error = Some(format!("Failed to write local file: {}", e));
            return result;
        }
    }

    // 12. Upload if not skip_upload
    if !skip_upload {
        let timestamp = chrono_lite_timestamp();
        let filename = format!(
            "{}_{}.automerge",
            sanitize_title(&bin_file.title),
            timestamp
        );
        if let Err(e) = client
            .upload_sb_file(storyboard_id, automerge_binary, &filename)
            .await
        {
            result.error = Some(format!("Upload failed: {}", e));
            return result;
        }
    }

    result.success = true;
    result
}

/// Reconstruct the full storyboard JSON by combining outer fields with data
fn reconstruct_storyboard_json(
    bin_file: &BinFile,
    data_value: &Value,
) -> Result<String, serde_json::Error> {
    // Build the full object
    let full = serde_json::json!({
        "id": bin_file.id,
        "title": bin_file.title,
        "description": bin_file.description,
        "scriptContent": bin_file.script_content,
        "createdAt": bin_file.created_at,
        "lastUpdated": bin_file.last_updated,
        "status": bin_file.status,
        "currentStage": bin_file.current_stage,
        "scriptFiles": bin_file.script_files,
        "driveFileIds": bin_file.drive_file_ids,
        "numShots": bin_file.num_shots,
        "thumbnailImage": bin_file.thumbnail_image,
        "lastSyncedSha": bin_file.last_synced_sha,
        "encryptedByEmail": bin_file.encrypted_by_email,
        "data": data_value
    });

    serde_json::to_string(&full)
}

/// Sanitize title for filename
fn sanitize_title(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

/// Generate a simple timestamp string
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_millis())
}
