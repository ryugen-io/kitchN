use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Fragment {
    #[serde(alias = "package")]
    pub meta: FragmentManifest,
    #[serde(default)]
    pub templates: Vec<Template>,
    #[serde(default)]
    pub files: Vec<Template>,
    #[serde(default)]
    pub hooks: Hooks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FragmentManifest {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub repository: Option<String>,
    pub license: Option<String>,
    
    // Legacy support (optional, but deprecated)
    pub id: Option<String>,
}

impl FragmentManifest {
    pub fn id(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Template {
    pub target: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Hooks {
    pub reload: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_deserialization() {
        let toml = r#"
            [package]
            name = "test.frag"
            version = "0.0.1"
            authors = ["Tester"]
            description = "A test fragment"

            [[templates]]
            target = "~/.config/test"
            content = "Hello {{ name }}"

            [hooks]
            reload = "echo reload"
        "#;

        let pkg: Fragment = toml::from_str(toml).unwrap();
        assert_eq!(pkg.meta.name, "test.frag");
        assert_eq!(pkg.meta.version, "0.0.1");
        assert_eq!(pkg.templates.len(), 1);
        assert_eq!(pkg.templates[0].target, "~/.config/test");
        assert_eq!(pkg.hooks.reload.unwrap(), "echo reload");
    }

    #[test]
    fn test_fragment_missing_required_fields() {
        let toml = r#"
            [package]
            name = "minimal"
            # Missing version, authors, description
        "#;
        let res: Result<Fragment, _> = toml::from_str(toml);
        assert!(res.is_err());
    }

    #[test]
    fn test_legacy_alias() {
        let toml = r#"
            [meta] # Should work as alias for [package]
            name = "legacy"
            version = "1.0"
            authors = ["Me"]
            description = "Legacy test"
        "#;
        let pkg: Fragment = toml::from_str(toml).unwrap();
        assert_eq!(pkg.meta.name, "legacy");
    }
}
