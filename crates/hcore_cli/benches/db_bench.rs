use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use hcore_lib::db::FragmentsDb;
use hcore_lib::fragment::{Fragment, FragmentManifest};
use tempfile::NamedTempFile;

fn create_dummy_fragment(id: usize) -> Fragment {
    Fragment {
        meta: FragmentManifest {
            name: format!("pkg_{}", id),
            description: "Benchmark fragment".to_string(),
            version: "1.0.0".to_string(),
            authors: vec![],
            repository: None,
            license: None,
            id: None,
        },
        templates: vec![],
        files: vec![],
        hooks: Default::default(),
    }
}

pub fn db_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragments_db");

    // Bench: Save DB (Serialization)
    group.bench_function("save_100_fragments", |b| {
        b.iter_batched(
            || {
                let file = NamedTempFile::new().unwrap();
                let path = file.path().to_path_buf();
                let mut db = FragmentsDb::load(&path).unwrap();
                for i in 0..100 {
                    db.install(create_dummy_fragment(i)).unwrap();
                }
                (db, file) // keep file alive
            },
            |(db, _file)| {
                db.save().unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    // Bench: Load DB (Deserialization)
    group.bench_function("load_100_fragments", |b| {
        // Setup: Create a pre-filled DB file
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        {
            let mut db = FragmentsDb::load(&path).unwrap();
            for i in 0..100 {
                db.install(create_dummy_fragment(i)).unwrap();
            }
            db.save().unwrap();
        }

        b.iter(|| {
            FragmentsDb::load(&path).unwrap();
        });
    });

    // Bench: Install (Ingestion)
    group.bench_function("install_single", |b| {
        b.iter_batched(
            || {
                let file = NamedTempFile::new().unwrap();
                let path = file.path().to_path_buf();
                let db = FragmentsDb::load(&path).unwrap();
                let frag = create_dummy_fragment(999);
                (db, frag, file)
            },
            |(mut db, frag, _file)| {
                db.install(frag).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, db_benchmarks);
criterion_main!(benches);
