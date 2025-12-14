use k_lib::db::Pantry;
use k_lib::ingredient::{Ingredient, IngredientManifest};
use tempfile::NamedTempFile;

fn create_ingredient(name: &str) -> Ingredient {
    Ingredient {
        meta: IngredientManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            authors: vec![],
            description: "".to_string(),
            repository: None,
            license: None,
        },
        templates: vec![],
        files: vec![],
        hooks: Default::default(),
    }
}

#[test]
fn test_db_persistence() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();

    // 1. Create and populate
    {
        let mut db = Pantry::load(path).unwrap();
        db.store(create_ingredient("theme_dark")).unwrap();
        db.store(create_ingredient("icon_pack")).unwrap();
        db.save().unwrap();
    }

    // 2. Load and verify
    {
        let db = Pantry::load(path).unwrap();
        let list = db.list();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|f| f.meta.name == "theme_dark"));
        assert!(list.iter().any(|f| f.meta.name == "icon_pack"));
    }
}

#[test]
fn test_install_remove() {
    let file = NamedTempFile::new().unwrap();
    let mut db = Pantry::load(file.path()).unwrap();

    // Install
    db.store(create_ingredient("obsolete")).unwrap();
    assert_eq!(db.list().len(), 1);

    // Remove
    let removed = db.discard("obsolete");
    assert!(removed.is_some());
    assert_eq!(db.list().len(), 0);

    // Remove non-existent
    assert!(db.discard("ghost").is_none());
}

#[test]
fn test_update_fragment() {
    let file = NamedTempFile::new().unwrap();
    let mut db = Pantry::load(file.path()).unwrap();

    // Initial install v1
    let mut f1 = create_ingredient("app");
    f1.meta.version = "1.0.0".to_string();
    db.store(f1).unwrap();

    assert_eq!(db.list()[0].meta.version, "1.0.0");

    // Update v2
    let mut f2 = create_ingredient("app");
    f2.meta.version = "2.0.0".to_string();
    db.store(f2).unwrap();

    assert_eq!(db.list()[0].meta.version, "2.0.0");
}
