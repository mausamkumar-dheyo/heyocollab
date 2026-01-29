//! Storyboard migration CLI tool
//!
//! Migrates storyboard files from encrypted .bin format to Automerge format.
//!
//! Usage:
//!   sb-migrate --base-url https://api.heyo.com --token "..." [OPTIONS]

mod client;
mod compression;
mod crypto;
mod migration;

// Re-use input and transform from json2automerge
#[path = "../json2automerge/input.rs"]
mod input;
#[path = "../json2automerge/transform.rs"]
mod transform;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sb-migrate",
    about = "Migrate storyboard files from encrypted .bin to Automerge format",
    version
)]
struct Args {
    /// Backend API base URL
    #[arg(short = 'b', long)]
    base_url: String,

    /// Auth token (or set HEYO_AUTH_TOKEN env var)
    #[arg(short = 't', long, env = "HEYO_AUTH_TOKEN")]
    token: String,

    /// Read auth token from file
    #[arg(long)]
    token_file: Option<PathBuf>,

    /// Specific storyboard IDs to migrate
    #[arg(short = 'i', long)]
    ids: Vec<String>,

    /// Read storyboard IDs from file (one per line)
    #[arg(long)]
    ids_file: Option<PathBuf>,

    /// Output directory for local backup
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,

    /// Download and convert only, don't upload
    #[arg(long)]
    skip_upload: bool,

    /// List storyboards without processing
    #[arg(long)]
    dry_run: bool,

    /// Re-migrate even if .automerge exists
    #[arg(long)]
    force: bool,

    /// Stop on first error
    #[arg(long)]
    abort_on_error: bool,

    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Resolve token
    let token = if let Some(token_file) = &args.token_file {
        std::fs::read_to_string(token_file)?.trim().to_string()
    } else {
        args.token.clone()
    };

    if token.is_empty() {
        anyhow::bail!("Auth token is required. Use --token or set HEYO_AUTH_TOKEN env var.");
    }

    // Create output directory if specified
    if let Some(ref dir) = args.output_dir {
        std::fs::create_dir_all(dir)?;
    }

    // Create client
    let client = client::HeyoClient::new(&args.base_url, &token)?;

    // Get storyboard list
    println!("Fetching storyboard list from {}...", args.base_url);
    let storyboards = client.list_storyboards().await?;
    println!("Found {} storyboards", storyboards.len());

    // Resolve target IDs
    let mut target_ids: Vec<String> = if !args.ids.is_empty() {
        args.ids.clone()
    } else if let Some(ref ids_file) = args.ids_file {
        std::fs::read_to_string(ids_file)?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect()
    } else {
        storyboards.iter().map(|s| s.id.clone()).collect()
    };

    // Filter to only valid IDs if specific IDs were provided
    if !args.ids.is_empty() || args.ids_file.is_some() {
        let valid_ids: std::collections::HashSet<_> =
            storyboards.iter().map(|s| s.id.as_str()).collect();
        let original_count = target_ids.len();
        target_ids.retain(|id| valid_ids.contains(id.as_str()));
        if target_ids.len() < original_count {
            println!(
                "Warning: {} of {} specified IDs not found",
                original_count - target_ids.len(),
                original_count
            );
        }
    }

    // Dry run - just list
    if args.dry_run {
        println!("\nDry run - {} storyboards would be migrated:", target_ids.len());
        for id in &target_ids {
            let sb = storyboards.iter().find(|s| s.id == *id);
            if let Some(sb) = sb {
                println!("  {} - {}", sb.id, sb.title);
            } else {
                println!("  {} - (not found)", id);
            }
        }
        return Ok(());
    }

    if target_ids.is_empty() {
        println!("No storyboards to migrate.");
        return Ok(());
    }

    // Progress bar
    let pb = ProgressBar::new(target_ids.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Process each storyboard
    let mut results = Vec::new();
    for id in &target_ids {
        let title = storyboards
            .iter()
            .find(|s| s.id == *id)
            .map(|s| s.title.as_str())
            .unwrap_or("Unknown");

        pb.set_message(format!("{}", title));

        let result = migration::migrate_storyboard(
            &client,
            id,
            args.skip_upload,
            args.output_dir.as_deref(),
            args.force,
        )
        .await;

        if args.verbose || !result.success {
            if result.success {
                if result.skipped {
                    println!("SKIP: {} ({}) - already migrated", result.storyboard_id, result.title);
                } else {
                    println!(
                        "OK: {} ({}) - {} -> {} bytes ({:.1}x compression)",
                        result.storyboard_id,
                        result.title,
                        result.input_size,
                        result.output_size,
                        if result.output_size > 0 {
                            result.input_size as f64 / result.output_size as f64
                        } else {
                            0.0
                        }
                    );
                }
            } else {
                eprintln!(
                    "FAIL: {} ({}) - {}",
                    result.storyboard_id,
                    result.title,
                    result.error.as_deref().unwrap_or("Unknown error")
                );
            }
        }

        if args.abort_on_error && !result.success {
            pb.finish_with_message("Aborted on error");
            return Err(anyhow::anyhow!(
                "Migration aborted: {}",
                result.error.unwrap_or_default()
            ));
        }

        results.push(result);
        pb.inc(1);
    }

    pb.finish_with_message("Done");

    // Summary
    let succeeded = results.iter().filter(|r| r.success && !r.skipped).count();
    let skipped = results.iter().filter(|r| r.skipped).count();
    let failed = results.iter().filter(|r| !r.success).count();

    let total_input: usize = results.iter().map(|r| r.input_size).sum();
    let total_output: usize = results.iter().map(|r| r.output_size).sum();

    println!("\n========================================");
    println!("Migration Summary:");
    println!("========================================");
    println!("  Succeeded: {}", succeeded);
    println!("  Skipped:   {}", skipped);
    println!("  Failed:    {}", failed);
    println!("  Total:     {}", results.len());
    println!();
    if total_output > 0 {
        println!(
            "  Total size: {} -> {} bytes ({:.1}x compression)",
            total_input,
            total_output,
            total_input as f64 / total_output as f64
        );
    }

    if failed > 0 {
        println!("\nFailed storyboards:");
        for r in results.iter().filter(|r| !r.success) {
            println!(
                "  {} ({}) - {}",
                r.storyboard_id,
                r.title,
                r.error.as_deref().unwrap_or("Unknown")
            );
        }
        std::process::exit(1);
    }

    Ok(())
}
