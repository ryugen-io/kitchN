use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use k_lib::db::Pantry;
use k_lib::ingredient::{Ingredient, IngredientManifest};
use tempfile::NamedTempFile;

fn create_dummy_ingredient(id: usize) -> Ingredient {
    Ingredient {
        meta: IngredientManifest {
            name: format!("pkg_{}", id),
            description: "Benchmark ingredient".to_string(),
            version: "1.0.0".to_string(),
            authors: vec![],
            repository: None,
            license: None,
        },
        templates: vec![],
        files: vec![],
        hooks: Default::default(),
    }
}

pub fn db_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("pantry_db");

    // Bench: Save DB (Serialization)
    group.bench_function("save_100_ingredients", |b| {
        b.iter_batched(
            || {
                let file = NamedTempFile::new().unwrap();
                let path = file.path().to_path_buf();
                let mut db = Pantry::load(&path).unwrap();
                for i in 0..100 {
                    db.store(create_dummy_ingredient(i)).unwrap();
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
    group.bench_function("load_100_ingredients", |b| {
        // Setup: Create a pre-filled DB file
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        {
            let mut db = Pantry::load(&path).unwrap();
            for i in 0..100 {
                db.store(create_dummy_ingredient(i)).unwrap();
            }
            db.save().unwrap();
        }

        b.iter(|| {
            Pantry::load(&path).unwrap();
        });
    });

    // Bench: Install (Ingestion)
    group.bench_function("store_single", |b| {
        b.iter_batched(
            || {
                let file = NamedTempFile::new().unwrap();
                let path = file.path().to_path_buf();
                let db = Pantry::load(&path).unwrap();
                let frag = create_dummy_ingredient(999);
                (db, frag, file)
            },
            |(mut db, frag, _file)| {
                db.store(frag).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, db_benchmarks);
criterion_main!(benches);
