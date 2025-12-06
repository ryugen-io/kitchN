use crate::config::HyprConfig;
use colored::CustomColor;

pub struct TagFactory;

impl TagFactory {
    pub fn create_tag(config: &HyprConfig, level: &str) -> String {
        // 1. Lookup Label
        let label = config
            .layout
            .labels
            .get(level)
            .map(|s| s.as_str())
            .unwrap_or(level);

        // 2. Transform
        let transformed = match config.layout.tag.transform.as_str() {
            "uppercase" => label.to_uppercase(),
            "lowercase" => label.to_lowercase(),
            "capitalize" => {
                let mut c = label.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
            _ => label.to_string(),
        };

        // 3. Pad
        let width = config.layout.tag.min_width;
        // Basic centering logic
        let len = transformed.chars().count();
        let padded = if len >= width {
            transformed
        } else {
            let total_padding = width - len;
            let left_pad = total_padding / 2;
            let right_pad = total_padding - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                transformed,
                " ".repeat(right_pad)
            )
        };

        // 4. Bracket
        format!(
            "{}{}{}",
            config.layout.tag.prefix, padded, config.layout.tag.suffix
        )
    }
}

pub struct ColorResolver;

impl ColorResolver {
    pub fn hex_to_color(hex: &str) -> CustomColor {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            CustomColor { r, g, b }
        } else {
            CustomColor {
                r: 255,
                g: 255,
                b: 255,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DictionaryConfig, HyprConfig, IconsConfig, LayoutConfig, LoggingConfig, StructureConfig,
        TagConfig, ThemeConfig, ThemeMeta, ThemeSettings,
    };
    use std::collections::HashMap;

    fn create_mock_config() -> HyprConfig {
        HyprConfig {
            theme: ThemeConfig {
                meta: ThemeMeta {
                    name: "Test".to_string(),
                },
                settings: ThemeSettings {
                    active_icons: "nerdfont".to_string(),
                },
                colors: HashMap::new(),
                fonts: HashMap::new(),
                include: None,
            },
            icons: IconsConfig {
                nerdfont: HashMap::new(),
                ascii: HashMap::new(),
                include: None,
            },
            layout: LayoutConfig {
                tag: TagConfig {
                    prefix: "[".to_string(),
                    suffix: "]".to_string(),
                    transform: "uppercase".to_string(),
                    min_width: 10,
                    alignment: "center".to_string(),
                },
                labels: HashMap::from([("error".to_string(), "Error".to_string())]),
                structure: StructureConfig {
                    terminal: "".to_string(),
                    file: "".to_string(),
                },
                logging: LoggingConfig {
                    base_dir: "".to_string(),
                    path_structure: "".to_string(),
                    filename_structure: "".to_string(),
                    timestamp_format: "".to_string(),
                    write_by_default: false,
                },
                include: None,
            },
            dictionary: DictionaryConfig {
                presets: HashMap::new(),
                include: None,
            },
        }
    }

    #[test]
    fn test_tag_creation_uppercase() {
        let config = create_mock_config();
        let tag = TagFactory::create_tag(&config, "error");
        assert_eq!(tag, "[  ERROR   ]");
    }

    #[test]
    fn test_tag_creation_lowercase() {
        let mut config = create_mock_config();
        config.layout.tag.transform = "lowercase".to_string();
        let tag = TagFactory::create_tag(&config, "error");
        assert_eq!(tag, "[  error   ]");
    }

    #[test]
    fn test_tag_creation_capitalize() {
        let mut config = create_mock_config();
        config.layout.tag.transform = "capitalize".to_string();
        // Label is "Error" in mock, so capitalize keeps it "Error"
        // Let's change label to "error" to test capitalization
        config
            .layout
            .labels
            .insert("error".to_string(), "error".to_string());
        let tag = TagFactory::create_tag(&config, "error");
        assert_eq!(tag, "[  Error   ]");
    }

    #[test]
    fn test_tag_creation_none() {
        let mut config = create_mock_config();
        config.layout.tag.transform = "none".to_string();
        let tag = TagFactory::create_tag(&config, "error");
        assert_eq!(tag, "[  Error   ]");
    }

    #[test]
    fn test_tag_padding_exact() {
        let mut config = create_mock_config();
        config.layout.tag.min_width = 5;
        let tag = TagFactory::create_tag(&config, "error");
        // "ERROR" is 5 chars. [ERROR]
        assert_eq!(tag, "[ERROR]");
    }

    #[test]
    fn test_tag_padding_overflow() {
        let mut config = create_mock_config();
        config.layout.tag.min_width = 3;
        let tag = TagFactory::create_tag(&config, "error");
        // "ERROR" is 5 chars, min 3. Should not truncate.
        assert_eq!(tag, "[ERROR]");
    }

    #[test]
    fn test_color_resolver_valid() {
        let color = ColorResolver::hex_to_color("#ff0000");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        let color = ColorResolver::hex_to_color("00ff00");
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);

        let color = ColorResolver::hex_to_color("#0000ff");
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_color_resolver_invalid() {
        // Invalid length
        let c = ColorResolver::hex_to_color("123");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);

        // Invalid chars (g is not hex) - from_str_radix will fail, unwrap_or(255)
        let c = ColorResolver::hex_to_color("gg0000");
        assert_eq!(c.r, 255);
    }
}
