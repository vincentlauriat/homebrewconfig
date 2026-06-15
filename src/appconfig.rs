use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Persisted user preferences for homebrewconfig itself (distinct from the
/// Homebrew variables it manages). Stored as TOML under the user config dir.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub theme: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("homebrewconfig").join("config.toml"))
}

/// Load preferences, falling back to defaults on any missing/invalid file.
pub fn load() -> AppConfig {
    config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Persist preferences to the user config dir, creating it if needed.
pub fn save(config: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("no config directory available")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
    }
    let toml = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, toml).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_theme_when_set() {
        let cfg = AppConfig {
            theme: Some("midnight".to_string()),
        };
        let toml = toml::to_string_pretty(&cfg).unwrap();
        assert!(toml.contains("theme = \"midnight\""));
    }

    #[test]
    fn omits_theme_when_unset() {
        let toml = toml::to_string_pretty(&AppConfig::default()).unwrap();
        assert!(!toml.contains("theme"));
    }

    #[test]
    fn round_trips_through_toml() {
        let toml = "theme = \"forest\"\n";
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.theme.as_deref(), Some("forest"));
    }
}
