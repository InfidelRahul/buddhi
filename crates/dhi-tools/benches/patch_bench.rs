use criterion::{criterion_group, criterion_main, Criterion};
use dhi_tools::patch_safety::{PatchProposal, PatchSafety};
use std::fs;
use std::path::PathBuf;

fn bench_patch_safety(c: &mut Criterion) {
    let test_file = PathBuf::from("bench_test.rs");
    fs::write(&test_file, "fn main() {}").unwrap();

    let proposal = PatchProposal {
        path: &test_file,
        original: "fn main() {}",
        replacement: "fn main() { println!(\"Hello\"); }",
        dry_run: true,
    };

    c.bench_function("patch_safety_dry_run", |b| {
        b.iter(|| {
            PatchSafety::apply(&proposal).unwrap();
        })
    });

    fs::remove_file(test_file).unwrap();
}

criterion_group!(benches, bench_patch_safety);
criterion_main!(benches);
