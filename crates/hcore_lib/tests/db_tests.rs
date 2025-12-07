use hcore_lib::db::FragmentsDb;
use hcore_lib::fragment::{Fragment, FragmentManifest};
use tempfile::NamedTempFile;

fn create_fragment(name: &str) -> Fragment {
    Fragment {
        meta: FragmentManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            authors: vec![],
            description: "".to_string(),
            repository: None,
            license: None,
            id: None,
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
        let mut db = FragmentsDb::load(path).unwrap();
        db.install(create_fragment("theme_dark")).unwrap();
        db.install(create_fragment("icon_pack")).unwrap();
        db.save().unwrap();
    }

    // 2. Load and verify
    {
        let db = FragmentsDb::load(path).unwrap();
        let list = db.list();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|f| f.meta.name == "theme_dark"));
        assert!(list.iter().any(|f| f.meta.name == "icon_pack"));
    }
}

#[test]
fn test_install_remove() {
    let file = NamedTempFile::new().unwrap();
    let mut db = FragmentsDb::load(file.path()).unwrap();

    // Install
    db.install(create_fragment("obsolete")).unwrap();
    assert_eq!(db.list().len(), 1);

    // Remove
    let removed = db.remove("obsolete");
    assert!(removed.is_some());
    assert_eq!(db.list().len(), 0);

    // Remove non-existent
    assert!(db.remove("ghost").is_none());
}

#[test]
fn test_update_fragment() {
    let file = NamedTempFile::new().unwrap();
    let mut db = FragmentsDb::load(file.path()).unwrap();

    // Initial install v1
    let mut f1 = create_fragment("app");
    f1.meta.version = "1.0.0".to_string();
    db.install(f1).unwrap();

    assert_eq!(db.list()[0].meta.version, "1.0.0");

    // Update v2
    let mut f2 = create_fragment("app");
    f2.meta.version = "2.0.0".to_string();
    db.install(f2).unwrap();

    assert_eq!(db.list()[0].meta.version, "2.0.0");
}
