//! Benchmarks for the collaborative sequence manager.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use heyocollab::{SequenceManager, GenerationNode, GenerationSettings, OutputAsset};

fn bench_new(c: &mut Criterion) {
    c.bench_function("new", |b| {
        b.iter(|| {
            black_box(SequenceManager::new())
        })
    });
}

fn bench_create_node_simple(c: &mut Criterion) {
    c.bench_function("create_node_simple", |b| {
        let mut manager = SequenceManager::new();
        let mut i = 0u64;
        b.iter(|| {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_prompt("A beautiful sunset");
            manager.create_node(&id, node).unwrap();
            i += 1;
        })
    });
}

fn bench_create_node_full(c: &mut Criterion) {
    c.bench_function("create_node_full", |b| {
        let mut manager = SequenceManager::new();
        let mut i = 0u64;
        b.iter(|| {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_title("My Generation")
                .with_prompt("A beautiful sunset over the ocean with golden light")
                .with_negative_prompt("blurry, low quality, distorted")
                .with_notes("Test generation for benchmarking")
                .with_settings(
                    GenerationSettings::new()
                        .with_seed(42)
                        .with_cfg(7.5)
                        .with_num_steps(30)
                        .with_model("stable-diffusion-xl")
                        .with_width(1024)
                        .with_height(1024)
                )
                .with_output(
                    OutputAsset::new("https://example.com/image1.png")
                        .with_seed(42)
                        .with_selected(true)
                )
                .with_output(
                    OutputAsset::new("https://example.com/image2.png")
                        .with_seed(43)
                );
            manager.create_node(&id, node).unwrap();
            i += 1;
        })
    });
}

fn bench_create_and_append(c: &mut Criterion) {
    c.bench_function("create_and_append", |b| {
        let mut manager = SequenceManager::new();
        let mut i = 0u64;
        b.iter(|| {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_prompt("A beautiful sunset");
            manager.create_and_append(&id, node).unwrap();
            i += 1;
        })
    });
}

fn bench_splice_char(c: &mut Criterion) {
    c.bench_function("splice_prompt_char", |b| {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test", "t2i")
            .with_prompt("Hello");
        manager.create_and_append("test", node).unwrap();

        let mut pos = 5usize;
        b.iter(|| {
            manager.splice_prompt("test", pos, 0, "x").unwrap();
            pos += 1;
        })
    });
}

fn bench_splice_word(c: &mut Criterion) {
    c.bench_function("splice_prompt_word", |b| {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test", "t2i")
            .with_prompt("Hello world this is a test prompt for benchmarking");
        manager.create_and_append("test", node).unwrap();

        b.iter(|| {
            // Insert and delete - simulating word replacement
            let len = manager.get_text("test", "prompt").unwrap().len();
            if len > 10 {
                manager.splice_prompt("test", 6, 5, "universe").unwrap();
                manager.splice_prompt("test", 6, 8, "world").unwrap();
            }
        })
    });
}

fn bench_get_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_state");

    for num_nodes in [1, 10, 50, 100].iter() {
        let mut manager = SequenceManager::new();
        for i in 0..*num_nodes {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_prompt("A test prompt")
                .with_settings(GenerationSettings::new().with_seed(i as i64));
            manager.create_and_append(&id, node).unwrap();
        }

        // Clear cache to force hydration
        let bytes = manager.save();
        let mut manager = SequenceManager::from_bytes(&bytes).unwrap();

        group.bench_with_input(
            BenchmarkId::new("nodes", num_nodes),
            num_nodes,
            |b, _| {
                b.iter(|| {
                    // Force re-hydration by saving/loading
                    let bytes = manager.save();
                    let mut m = SequenceManager::from_bytes(&bytes).unwrap();
                    black_box(m.get_state().unwrap())
                })
            },
        );
    }
    group.finish();
}

fn bench_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("save");

    for num_nodes in [1, 10, 50].iter() {
        let mut manager = SequenceManager::new();
        for i in 0..*num_nodes {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_prompt("A test prompt for saving benchmark")
                .with_settings(GenerationSettings::new().with_seed(i as i64));
            manager.create_and_append(&id, node).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::new("nodes", num_nodes),
            num_nodes,
            |b, _| {
                b.iter(|| {
                    black_box(manager.save())
                })
            },
        );
    }
    group.finish();
}

fn bench_merge(c: &mut Criterion) {
    c.bench_function("merge_10_nodes", |b| {
        // Create base document with 10 nodes
        let mut base = SequenceManager::new();
        for i in 0..10 {
            let id = format!("node-{}", i);
            let node = GenerationNode::new(&id, "t2i")
                .with_prompt("A test prompt");
            base.create_and_append(&id, node).unwrap();
        }
        let base_bytes = base.save();

        b.iter(|| {
            let mut client_a = SequenceManager::from_bytes(&base_bytes).unwrap();
            let mut client_b = SequenceManager::from_bytes(&base_bytes).unwrap();

            // Make changes
            let node_a = GenerationNode::new("new-a", "t2i");
            client_a.create_and_append("new-a", node_a).unwrap();

            let node_b = GenerationNode::new("new-b", "t2i");
            client_b.create_and_append("new-b", node_b).unwrap();

            // Merge
            client_a.merge(&mut client_b).unwrap();
            black_box(&client_a);
        })
    });
}

fn bench_update_settings(c: &mut Criterion) {
    c.bench_function("update_settings_reconcile", |b| {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test", "t2i");
        manager.create_and_append("test", node).unwrap();

        let mut seed = 0i64;
        b.iter(|| {
            manager.update_settings("test", |settings| {
                settings.seed = Some(seed);
                settings.cfg = Some(7.5);
            }).unwrap();
            seed += 1;
        })
    });
}

fn bench_targeted_settings(c: &mut Criterion) {
    c.bench_function("set_setting_direct", |b| {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test", "t2i");
        manager.create_and_append("test", node).unwrap();

        let mut seed = 0i64;
        b.iter(|| {
            manager.set_setting_seed("test", Some(seed)).unwrap();
            manager.set_setting_cfg("test", Some(7.5)).unwrap();
            seed += 1;
        })
    });
}

fn bench_set_status(c: &mut Criterion) {
    c.bench_function("set_status_direct", |b| {
        let mut manager = SequenceManager::new();
        let node = GenerationNode::new("test", "t2i");
        manager.create_and_append("test", node).unwrap();

        let statuses = ["pending", "processing", "completed", "failed"];
        let mut i = 0usize;
        b.iter(|| {
            manager.set_status("test", statuses[i % 4]).unwrap();
            i += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_new,
    bench_create_node_simple,
    bench_create_node_full,
    bench_create_and_append,
    bench_splice_char,
    bench_splice_word,
    bench_get_state,
    bench_save,
    bench_merge,
    bench_update_settings,
    bench_targeted_settings,
    bench_set_status,
);

criterion_main!(benches);
