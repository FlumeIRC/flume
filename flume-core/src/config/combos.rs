//! User-defined color combination shortcuts.
//!
//! Users define combos in `[combos]` section of config.toml:
//!
//! ```toml
//! [combos]
//! alert = "%B%Cred,white"       # static: format string
//!
//! [combos.rainbow]              # dynamic: cycles colors per character
//! type = "cycle"
//! colors = ["red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink"]
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Top-level combos config section.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CombosConfig {
    #[serde(flatten)]
    pub combos: HashMap<String, ComboDefinition>,
}

/// A combo is either a static format string or a dynamic cycling definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ComboDefinition {
    /// Dynamic combo with a type and color list.
    Dynamic(DynamicCombo),
    /// Static combo: a format string using %B/%C shortcuts.
    Static(String),
}

/// A dynamic combo definition (e.g., cycling colors per character).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicCombo {
    /// The combo type. Currently only "cycle" is supported.
    #[serde(rename = "type")]
    pub combo_type: String,
    /// List of color names to cycle through.
    pub colors: Vec<String>,
}

/// Return the built-in default combos.
pub fn default_combos() -> HashMap<String, ComboDefinition> {
    let mut combos = HashMap::new();
    combos.insert(
        "rainbow".to_string(),
        ComboDefinition::Dynamic(DynamicCombo {
            combo_type: "cycle".to_string(),
            colors: vec![
                "red".into(),
                "orange".into(),
                "yellow".into(),
                "green".into(),
                "cyan".into(),
                "blue".into(),
                "purple".into(),
                "pink".into(),
            ],
        }),
    );
    combos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_static_combo() {
        let toml_str = r#"alert = "%B%Cred,white""#;
        let config: CombosConfig = toml::from_str(toml_str).unwrap();
        assert!(matches!(config.combos.get("alert"), Some(ComboDefinition::Static(s)) if s == "%B%Cred,white"));
    }

    #[test]
    fn deserialize_dynamic_combo() {
        let toml_str = r#"
[rainbow]
type = "cycle"
colors = ["red", "blue", "green"]
"#;
        let config: CombosConfig = toml::from_str(toml_str).unwrap();
        match config.combos.get("rainbow") {
            Some(ComboDefinition::Dynamic(d)) => {
                assert_eq!(d.combo_type, "cycle");
                assert_eq!(d.colors, vec!["red", "blue", "green"]);
            }
            other => panic!("Expected Dynamic, got {:?}", other),
        }
    }

    #[test]
    fn deserialize_mixed() {
        let toml_str = r#"
alert = "%B%Cred"
info = "%Ccyan"

[usa]
type = "cycle"
colors = ["red", "white", "blue"]
"#;
        let config: CombosConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.combos.len(), 3);
        assert!(matches!(config.combos.get("alert"), Some(ComboDefinition::Static(_))));
        assert!(matches!(config.combos.get("info"), Some(ComboDefinition::Static(_))));
        assert!(matches!(config.combos.get("usa"), Some(ComboDefinition::Dynamic(_))));
    }

    #[test]
    fn serialize_round_trip() {
        let mut combos = HashMap::new();
        combos.insert("alert".to_string(), ComboDefinition::Static("%B%Cred".to_string()));
        combos.insert(
            "rainbow".to_string(),
            ComboDefinition::Dynamic(DynamicCombo {
                combo_type: "cycle".to_string(),
                colors: vec!["red".into(), "blue".into()],
            }),
        );
        let config = CombosConfig { combos };
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CombosConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.combos.len(), 2);
    }

    #[test]
    fn default_combos_has_rainbow() {
        let combos = default_combos();
        assert!(matches!(combos.get("rainbow"), Some(ComboDefinition::Dynamic(_))));
    }
}
