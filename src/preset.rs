use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::app::{App, Setting, SettingKind};

/// A shareable snapshot of non-default settings, keyed by environment variable.
#[derive(Serialize, Deserialize)]
struct Preset {
    #[serde(default)]
    settings: BTreeMap<String, String>,
}

/// Serialize the app's non-default settings to a TOML preset.
pub fn export_preset(app: &App) -> Result<String, String> {
    let mut settings = BTreeMap::new();
    for s in &app.settings {
        if let Some(raw) = exported_raw(s) {
            settings.insert(s.env_var.to_string(), raw);
        }
    }
    let preset = Preset { settings };
    toml::to_string_pretty(&preset).map_err(|e| format!("failed to serialize preset: {}", e))
}

/// Parse a TOML preset into `(env_var, raw_value)` pairs.
pub fn parse_preset(content: &str) -> Result<Vec<(String, String)>, String> {
    let preset: Preset = toml::from_str(content).map_err(|e| format!("invalid preset: {}", e))?;
    Ok(preset.settings.into_iter().collect())
}

/// The raw environment value a setting would export, or `None` when it is at
/// its Homebrew default (and therefore omitted from a preset).
fn exported_raw(s: &Setting) -> Option<String> {
    match &s.kind {
        SettingKind::Bool { inverted } => {
            let present = if *inverted { !s.bool_val } else { s.bool_val };
            present.then(|| "1".to_string())
        }
        SettingKind::Str => (!s.str_val.is_empty()).then(|| s.str_val.clone()),
        SettingKind::Num => s.num_val.map(|n| n.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn export_omits_defaults_and_includes_changes() {
        let mut app = App::new();
        app.set_var("HOMEBREW_CLEANUP_MAX_AGE_DAYS", Some("30".to_string()))
            .unwrap();
        let toml = export_preset(&app).unwrap();
        assert!(toml.contains("HOMEBREW_CLEANUP_MAX_AGE_DAYS = \"30\""));
    }

    #[test]
    fn parse_round_trips_with_export() {
        let mut app = App::new();
        app.set_var("HOMEBREW_NO_ANALYTICS", Some("1".to_string()))
            .unwrap();
        let toml = export_preset(&app).unwrap();
        let pairs = parse_preset(&toml).unwrap();
        assert!(pairs
            .iter()
            .any(|(k, v)| k == "HOMEBREW_NO_ANALYTICS" && v == "1"));
    }

    #[test]
    fn parse_rejects_malformed_toml() {
        assert!(parse_preset("this is not = = toml").is_err());
    }

    #[test]
    fn parse_accepts_empty_preset() {
        assert_eq!(parse_preset("").unwrap().len(), 0);
    }
}
