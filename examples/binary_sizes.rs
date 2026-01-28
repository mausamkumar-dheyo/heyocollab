//! Binary size analysis for heyocollab documents.
//!
//! Run with: cargo run --release --example binary_sizes

use heyocollab::{GenerationNode, GenerationSettings, OutputAsset, SequenceManager};

fn create_realistic_node(id: &str, prompt_len: usize) -> GenerationNode {
    let prompt: String = (0..prompt_len)
        .map(|i| (b'a' + (i % 26) as u8) as char)
        .collect();

    GenerationNode::new(id, "text-to-image")
        .with_title("Generated Image")
        .with_prompt(&prompt)
        .with_negative_prompt("blurry, low quality, distorted, watermark")
        .with_notes("Test generation")
        .with_settings(
            GenerationSettings::new()
                .with_seed(42)
                .with_cfg(7.5)
                .with_num_steps(30)
                .with_model("stable-diffusion-xl-1.0")
                .with_width(1024)
                .with_height(1024),
        )
        .with_output(
            OutputAsset::new("https://cdn.example.com/images/gen-001-seed42.png").with_seed(42),
        )
        .with_output(
            OutputAsset::new("https://cdn.example.com/images/gen-001-seed43.png").with_seed(43),
        )
}

fn main() {
    println!("=== HeyoCollab Binary Size Analysis ===\n");

    // Compiled library sizes
    println!("## Compiled Library Sizes (release build)");
    println!("- libheyocollab.rlib: ~1.3 MB (static library)");
    println!("- libheyocollab.so:   ~536 KB (shared library)");
    println!();

    // Empty document
    let mut manager = SequenceManager::new();
    let empty_size = manager.save().len();
    println!("## Serialized Document Sizes\n");
    println!("| Nodes | Prompt Len | Binary Size | Per Node |");
    println!("|-------|------------|-------------|----------|");
    println!(
        "| 0     | -          | {} bytes   | -        |",
        empty_size
    );

    // Test various document sizes
    let test_cases = [
        (1, 50),    // 1 node, short prompt
        (1, 500),   // 1 node, long prompt
        (10, 100),  // 10 nodes, medium prompts
        (50, 100),  // 50 nodes
        (100, 100), // 100 nodes
    ];

    for (num_nodes, prompt_len) in test_cases {
        let mut manager = SequenceManager::new();

        for i in 0..num_nodes {
            let id = format!("node-{:04}", i);
            let node = create_realistic_node(&id, prompt_len);
            manager.create_and_append(&id, node).unwrap();
        }

        let binary = manager.save();
        let size = binary.len();
        let per_node = if num_nodes > 0 {
            (size - empty_size) / num_nodes
        } else {
            0
        };

        let size_str = if size > 1024 * 1024 {
            format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
        } else if size > 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        };

        let per_node_str = if per_node > 1024 {
            format!("{:.2} KB", per_node as f64 / 1024.0)
        } else {
            format!("{} bytes", per_node)
        };

        println!(
            "| {:5} | {:10} | {:>11} | {:>8} |",
            num_nodes, prompt_len, size_str, per_node_str
        );
    }

    println!();

    // Incremental sync message sizes
    println!("## Sync Message Sizes\n");
    println!("| Operation | Message Size |");
    println!("|-----------|--------------|");

    // Create base document
    let mut base = SequenceManager::new();
    let node = create_realistic_node("base-node", 100);
    base.create_and_append("base-node", node).unwrap();
    let base_heads = base.get_heads();

    // Settings change
    base.set_setting_seed("base-node", Some(999)).unwrap();
    let setting_sync = base
        .generate_sync_message(&base_heads)
        .map(|b| b.len())
        .unwrap_or(0);
    println!("| Setting change | {} bytes |", setting_sync);

    let heads_after_setting = base.get_heads();

    // Add new node
    let new_node = create_realistic_node("new-node", 100);
    base.create_and_append("new-node", new_node).unwrap();
    let new_node_sync = base
        .generate_sync_message(&heads_after_setting)
        .map(|b| b.len())
        .unwrap_or(0);
    println!("| Add new node (100 char prompt) | {} bytes |", new_node_sync);

    println!();

    // JSON export size comparison
    println!("## JSON vs Binary Comparison\n");
    let mut manager = SequenceManager::new();
    for i in 0..10 {
        let id = format!("node-{:04}", i);
        let node = create_realistic_node(&id, 200);
        manager.create_and_append(&id, node).unwrap();
    }

    let binary_size = manager.save().len();
    let state = manager.get_state().unwrap();

    // Approximate JSON size by serializing what we can
    let mut json_approx = 0;
    for (id, node) in &state.generations {
        json_approx += id.len();
        json_approx += node.to_json_value().to_string().len();
    }
    json_approx += state.sequence_order.iter().map(|s| s.len() + 3).sum::<usize>();

    println!("| Format | Size (10 nodes, 200 char prompts) |");
    println!("|--------|-----------------------------------|");
    println!(
        "| Binary | {:.2} KB |",
        binary_size as f64 / 1024.0
    );
    println!(
        "| JSON (approx) | {:.2} KB |",
        json_approx as f64 / 1024.0
    );
    println!(
        "| Ratio | {:.2}x smaller |",
        json_approx as f64 / binary_size as f64
    );

}
