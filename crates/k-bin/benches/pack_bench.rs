use criterion::{Criterion, criterion_group, criterion_main};
use k_lib::packager::{pack, unpack};
use std::fs;
use tempfile::tempdir;

fn bench_pack(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let output_file = dir.path().join("bench.bag");

    fs::create_dir_all(&source_dir).unwrap();
    // Create 100 dummy fragments
    for i in 0..100 {
        fs::write(
            source_dir.join(format!("test_{}.ing", i)),
            format!("[package]\nname = \"test_{}\"\nversion=\"1.0\"\nauthors=[]\ndescription=\"bench\"\n", i),
        )
        .unwrap();
    }

    c.bench_function("pack 100 ingredients", |b| {
        b.iter(|| pack(&source_dir, &output_file).unwrap())
    });
}

fn bench_unpack(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let output_file = dir.path().join("bench.bag");
    let unpack_dir = dir.path().join("unpacked");

    fs::create_dir_all(&source_dir).unwrap();
    for i in 0..100 {
        fs::write(
            source_dir.join(format!("test_{}.ing", i)),
            format!("[package]\nname = \"test_{}\"\nversion=\"1.0\"\nauthors=[]\ndescription=\"bench\"\n", i),
        )
        .unwrap();
    }
    pack(&source_dir, &output_file).unwrap();

    c.bench_function("unpack 100 ingredients", |b| {
        b.iter(|| {
            let _ = fs::remove_dir_all(&unpack_dir); // Cleanup
            unpack(&output_file, &unpack_dir).unwrap()
        })
    });
}

criterion_group!(benches, bench_pack, bench_unpack);
criterion_main!(benches);
