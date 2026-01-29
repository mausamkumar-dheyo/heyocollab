//! CLI tool to convert storyboard JSON to Automerge binary format.
//!
//! Usage:
//!   json2automerge --input storyboard.json [--output storyboard.automerge] [--validate] [--stats]

mod input;
mod transform;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use heyocollab::storyboard::{StoryboardManager, StoryboardRoot};
use input::InputStoryboard;

#[derive(Parser, Debug)]
#[command(
    name = "json2automerge",
    about = "Convert storyboard JSON to Automerge binary format",
    version
)]
struct Args {
    /// Input JSON file path (decrypted storyboard)
    #[arg(short, long)]
    input: PathBuf,

    /// Output file path (defaults to input path with .automerge extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Validate output by hydrating back to structs
    #[arg(long, default_value = "false")]
    validate: bool,

    /// Print statistics about the conversion
    #[arg(long, default_value = "false")]
    stats: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Validate input exists
    let input_path = &args.input;
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {}", input_path.display());
    }

    // 2. Read JSON file
    let json_content =
        std::fs::read_to_string(input_path).context("Failed to read input file")?;

    // 3. Parse JSON to input structs
    let input: InputStoryboard =
        serde_json::from_str(&json_content).context("Failed to parse JSON")?;

    // Store some stats before transformation
    let input_id = input.id.clone();
    let input_title = input.title.clone();
    let num_characters = input.data.processing_stages.characters.len();
    let num_props = input.data.processing_stages.props.len();
    let num_sets = input.data.processing_stages.sets.len();
    let num_scenes = input.data.scenes.len();
    let total_shots: usize = input.data.scenes.iter().map(|s| s.shots.len()).sum();

    // 4. Transform to Rust model
    let root: StoryboardRoot = input.into();

    // 5. Create Automerge document
    let mut manager = StoryboardManager::new();
    manager
        .update_state(|state| {
            *state = root;
        })
        .context("Failed to update Automerge document state")?;

    // 6. Save to binary
    let binary = manager.save();

    // 7. Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = input_path.clone();
        path.set_extension("automerge");
        path
    });

    // 8. Write output
    std::fs::write(&output_path, &binary).context("Failed to write output file")?;

    // 9. Optional validation
    if args.validate {
        let mut loaded =
            StoryboardManager::from_bytes(&binary).context("Failed to load binary for validation")?;
        let hydrated = loaded
            .get_state()
            .context("Failed to hydrate for validation")?;

        // Basic validation - check key counts match
        if hydrated.scenes.len() != num_scenes {
            anyhow::bail!(
                "Validation failed: scene count mismatch (expected {}, got {})",
                num_scenes,
                hydrated.scenes.len()
            );
        }
        if hydrated.processing_stages.characters.len() != num_characters {
            anyhow::bail!(
                "Validation failed: character count mismatch (expected {}, got {})",
                num_characters,
                hydrated.processing_stages.characters.len()
            );
        }
        if hydrated.processing_stages.props.len() != num_props {
            anyhow::bail!(
                "Validation failed: prop count mismatch (expected {}, got {})",
                num_props,
                hydrated.processing_stages.props.len()
            );
        }
        if hydrated.processing_stages.sets.len() != num_sets {
            anyhow::bail!(
                "Validation failed: set count mismatch (expected {}, got {})",
                num_sets,
                hydrated.processing_stages.sets.len()
            );
        }

        // Count total shots in hydrated
        let hydrated_shots: usize = hydrated.scenes.values().map(|s| s.shots.len()).sum();
        if hydrated_shots != total_shots {
            anyhow::bail!(
                "Validation failed: shot count mismatch (expected {}, got {})",
                total_shots,
                hydrated_shots
            );
        }

        println!("✓ Validation passed!");
    }

    // 10. Optional stats
    if args.stats {
        println!();
        println!("Conversion statistics:");
        println!("  Storyboard ID: {}", input_id);
        println!("  Title: {}", input_title);
        println!();
        println!("  Input JSON:    {:>10} bytes", json_content.len());
        println!("  Output binary: {:>10} bytes", binary.len());
        println!(
            "  Compression:   {:>10.2}x",
            json_content.len() as f64 / binary.len() as f64
        );
        println!();
        println!("  Characters: {}", num_characters);
        println!("  Props:      {}", num_props);
        println!("  Sets:       {}", num_sets);
        println!("  Scenes:     {}", num_scenes);
        println!("  Shots:      {}", total_shots);
    }

    println!();
    println!(
        "Successfully converted {} → {}",
        input_path.display(),
        output_path.display()
    );

    Ok(())
}
