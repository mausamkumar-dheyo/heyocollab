//! Advanced Stress Test Suite for HeyoCollab (Autosurgeon)
//!
//! Covers: N-User Scalability and Serialization Overhead
//!
//! Run with: cargo run --release --example stress_test

use heyocollab::{GenerationNode, SequenceManager};
use std::time::Instant;

fn main() {
    println!("========================================");
    println!(" HeyoCollab (Autosurgeon) Stress Suite");
    println!("========================================\n");

    test_n_user_scalability(50);
    test_serialization_overhead();
}

// -----------------------------------------------------------------------------
// 1. N-User Scalability (The "Separate Generations" Test)
// -----------------------------------------------------------------------------
fn test_n_user_scalability(users: usize) {
    println!("Test: Scalability ({} concurrent users creating separate generations)", users);

    // 1. Init "Server" document
    let mut server = SequenceManager::new();

    let start = Instant::now();

    // 2. Simulate each user creating their own generation node
    let mut total_merges = 0;

    for i in 0..users {
        // Each user forks from current server state
        let server_bytes = server.save();
        let mut client = SequenceManager::from_bytes(&server_bytes).unwrap();

        // User creates their own generation with their own prompt
        let id = format!("user-{}", i);
        let node = GenerationNode::new(&id, "t2i")
            .with_prompt(&format!("User {} prompt: A beautiful sunset over mountains", i))
            .with_title(&format!("User {} Generation", i));

        client.create_and_append(&id, node).unwrap();

        // Server receives update via merge
        server.merge(&mut client).unwrap();
        total_merges += 1;
    }

    let duration = start.elapsed();
    println!("   Total Merges:     {}", total_merges);
    println!("   Total Time:       {:?}", duration);
    println!(
        "   Server Capacity:  {:.0} merges/sec",
        total_merges as f64 / duration.as_secs_f64()
    );

    // Validate consistency
    let state = server.get_state().unwrap();
    println!(
        "   Total Generations: {} (Expected: {})",
        state.len(),
        users
    );

    // Verify all user generations are present
    let user_count = (0..users)
        .filter(|i| {
            let id = format!("user-{}", i);
            state.generations.contains_key(&id)
        })
        .count();

    println!("   Users Preserved:  {}/{}", user_count, users);

    // Sample a few to verify content
    if let Some(node) = state.generations.get("user-0") {
        println!("   Sample (User 0):  \"{}\"", node.prompt_str());
    }

    println!("   [Analysis]: Each user creates separate generations. No conflicts possible.\n");
}

// -----------------------------------------------------------------------------
// 2. Serialization Overhead (WASM Proxy)
// -----------------------------------------------------------------------------
fn test_serialization_overhead() {
    println!("Test: Serialization (Proxy for WASM Boundary)");

    let mut manager = SequenceManager::new();

    // Create a "Heavy" document with 100 nodes
    for i in 0..100 {
        let id = format!("node-{}", i);
        let node = GenerationNode::new(&id, "t2i")
            .with_prompt("A very long detailed prompt that simulates realistic usage data for benchmarking purposes...")
            .with_title(&format!("Generation {}", i))
            .with_negative_prompt("blurry, low quality, distorted, watermark, text");
        manager.create_and_append(&id, node).unwrap();
    }

    // Measure hydration (get_state)
    let start = Instant::now();
    let state = manager.get_state().unwrap();
    let hydrate_time = start.elapsed();

    // Measure JSON serialization
    let start = Instant::now();
    let mut json_parts = Vec::new();
    for (id, node) in &state.generations {
        json_parts.push((id.clone(), node.to_json_value()));
    }
    let _json = serde_json::to_string(&json_parts).unwrap();
    let json_time = start.elapsed();

    // Measure binary save
    let start = Instant::now();
    let binary = manager.save();
    let save_time = start.elapsed();

    // Measure binary load
    let start = Instant::now();
    let _ = SequenceManager::from_bytes(&binary).unwrap();
    let load_time = start.elapsed();

    println!("   Nodes:            100");
    println!("   Hydrate Time:     {:>8.2?}", hydrate_time);
    println!("   JSON Export:      {:>8.2?}", json_time);
    println!("   Binary Save:      {:>8.2?}", save_time);
    println!("   Binary Load:      {:>8.2?}", load_time);
    println!("   Binary Size:      {:>8} bytes ({:.1} KB)", binary.len(), binary.len() as f64 / 1024.0);
    println!("   Bytes per Node:   {:.0} bytes", binary.len() as f64 / 100.0);
    println!("   [Analysis]: If > 16ms, UI may freeze during load/save.\n");
}
