use std::env;
use std::path::PathBuf;

use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingKind {
    Bool { inverted: bool },
    Str,
    Num,
}

#[derive(Debug, Clone)]
pub struct Setting {
    pub name: &'static str,
    pub env_var: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
    pub bool_val: bool,
    pub str_val: String,
    pub num_val: Option<u32>,
    pub category: &'static str,
    pub modified: bool,
}

impl Setting {
    pub fn value_display(&self) -> String {
        match &self.kind {
            SettingKind::Bool { .. } => {
                if self.bool_val {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                }
            }
            SettingKind::Str => {
                if self.str_val.is_empty() {
                    "(default)".to_string()
                } else {
                    self.str_val.clone()
                }
            }
            SettingKind::Num => match self.num_val {
                Some(n) => n.to_string(),
                None => "(default)".to_string(),
            },
        }
    }

    pub fn is_feature_on(&self) -> bool {
        self.bool_val
    }

    fn read_from_env(&mut self) {
        let raw = env::var(self.env_var).ok();
        self.apply_env_value(raw);
    }

    /// Pure conversion from a raw environment value to this setting's typed
    /// value. Split out from `read_from_env` so it can be unit-tested without
    /// mutating the process environment.
    fn apply_env_value(&mut self, raw: Option<String>) {
        match &self.kind {
            SettingKind::Bool { inverted } => {
                let is_set = raw.is_some();
                self.bool_val = if *inverted { !is_set } else { is_set };
            }
            SettingKind::Str => {
                self.str_val = raw.unwrap_or_default();
            }
            SettingKind::Num => {
                self.num_val = raw.and_then(|v| v.trim().parse().ok());
            }
        }
        self.modified = false;
    }
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Editing,
    Confirming,
}

pub struct App {
    pub settings: Vec<Setting>,
    pub selected: usize,
    pub list_state: ListState,
    pub mode: Mode,
    pub input: String,
    pub cursor_pos: usize,
    pub shell_profile: PathBuf,
    pub message: Option<(String, bool)>,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        let mut settings = Self::build_settings();
        for setting in &mut settings {
            setting.read_from_env();
        }
        let mut list_state = ListState::default();
        list_state.select(Some(Self::compute_list_index(&settings, 0)));
        App {
            settings,
            selected: 0,
            list_state,
            mode: Mode::Normal,
            input: String::new(),
            cursor_pos: 0,
            shell_profile: Self::detect_shell_profile(),
            message: None,
            show_help: false,
        }
    }

    pub fn toggle_current(&mut self) {
        let setting = &mut self.settings[self.selected];
        if matches!(setting.kind, SettingKind::Bool { .. }) {
            setting.bool_val = !setting.bool_val;
            setting.modified = true;
        }
    }

    pub fn start_editing(&mut self) {
        let setting = &self.settings[self.selected];
        match &setting.kind {
            SettingKind::Str => {
                self.input = setting.str_val.clone();
            }
            SettingKind::Num => {
                self.input = setting.num_val.map(|n| n.to_string()).unwrap_or_default();
            }
            SettingKind::Bool { .. } => return,
        }
        self.cursor_pos = self.input.chars().count();
        self.mode = Mode::Editing;
    }

    pub fn confirm_edit(&mut self) {
        let setting = &mut self.settings[self.selected];
        match &setting.kind {
            SettingKind::Str => {
                let trimmed = self.input.trim().to_string();
                if trimmed != setting.str_val {
                    setting.str_val = trimmed;
                    setting.modified = true;
                }
            }
            SettingKind::Num => {
                let trimmed = self.input.trim();
                let new_val = if trimmed.is_empty() {
                    None
                } else {
                    match trimmed.parse::<u32>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            self.message = Some(("Invalid number".to_string(), true));
                            return;
                        }
                    }
                };
                if new_val != setting.num_val {
                    setting.num_val = new_val;
                    setting.modified = true;
                }
            }
            SettingKind::Bool { .. } => {}
        }
        self.cancel_edit();
    }

    pub fn cancel_edit(&mut self) {
        self.mode = Mode::Normal;
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn select_next(&mut self) {
        if self.settings.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.settings.len();
        self.sync_list_state();
    }

    pub fn select_prev(&mut self) {
        if self.settings.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.settings.len() - 1
        } else {
            self.selected - 1
        };
        self.sync_list_state();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.settings.iter().any(|s| s.modified)
    }

    pub fn reset(&mut self) {
        for setting in &mut self.settings {
            setting.read_from_env();
        }
        self.sync_list_state();
        self.message = Some(("Reset to current environment".to_string(), false));
    }

    fn build_settings() -> Vec<Setting> {
        let make = |name, env_var, description, kind, category| Setting {
            name,
            env_var,
            description,
            kind,
            bool_val: false,
            str_val: String::new(),
            num_val: None,
            category,
            modified: false,
        };
        vec![
            make(
                "Analytics",
                "HOMEBREW_NO_ANALYTICS",
                "Disable sending analytics to Google Analytics",
                SettingKind::Bool { inverted: true },
                "Privacy",
            ),
            make(
                "Env Hints",
                "HOMEBREW_NO_ENV_HINTS",
                "Disable hints about Homebrew environment variables",
                SettingKind::Bool { inverted: true },
                "Privacy",
            ),
            make(
                "Auto Update",
                "HOMEBREW_NO_AUTO_UPDATE",
                "Disable automatic brew update before install/upgrade/tap",
                SettingKind::Bool { inverted: true },
                "Updates",
            ),
            make(
                "Install Upgrade",
                "HOMEBREW_NO_INSTALL_UPGRADE",
                "Disable automatic upgrade of outdated formulae on install",
                SettingKind::Bool { inverted: true },
                "Updates",
            ),
            make(
                "Install Cleanup",
                "HOMEBREW_NO_INSTALL_CLEANUP",
                "Disable automatic cleanup after install/upgrade",
                SettingKind::Bool { inverted: true },
                "Updates",
            ),
            make(
                "Cleanup Age",
                "HOMEBREW_CLEANUP_MAX_AGE_DAYS",
                "Max age in days for cached files during cleanup (default: 120)",
                SettingKind::Num,
                "Updates",
            ),
            make(
                "Color",
                "HOMEBREW_NO_COLOR",
                "Disable colored output",
                SettingKind::Bool { inverted: true },
                "Display",
            ),
            make(
                "Emoji",
                "HOMEBREW_NO_EMOJI",
                "Disable emoji in output (e.g. the 🍺 after install)",
                SettingKind::Bool { inverted: true },
                "Display",
            ),
            make(
                "Verbose",
                "HOMEBREW_VERBOSE",
                "Print extra debugging information",
                SettingKind::Bool { inverted: false },
                "Display",
            ),
            make(
                "Debug",
                "HOMEBREW_DEBUG",
                "Print all debugging information",
                SettingKind::Bool { inverted: false },
                "Display",
            ),
            make(
                "Install Badge",
                "HOMEBREW_INSTALL_BADGE",
                "Character displayed during install (default: 🍺)",
                SettingKind::Str,
                "Display",
            ),
            make(
                "Cache",
                "HOMEBREW_CACHE",
                "Directory for cached downloads",
                SettingKind::Str,
                "Directories",
            ),
            make(
                "Cellar",
                "HOMEBREW_CELLAR",
                "Directory for installed formula files",
                SettingKind::Str,
                "Directories",
            ),
            make(
                "Logs",
                "HOMEBREW_LOGS",
                "Directory for brew logs",
                SettingKind::Str,
                "Directories",
            ),
            make(
                "Temp",
                "HOMEBREW_TEMP",
                "Temp directory used during builds",
                SettingKind::Str,
                "Directories",
            ),
            make(
                "Editor",
                "HOMEBREW_EDITOR",
                "Editor used by brew edit (defaults to $EDITOR)",
                SettingKind::Str,
                "Tools",
            ),
            make(
                "Curl Retries",
                "HOMEBREW_CURL_RETRIES",
                "Number of times to retry failed downloads (default: 3)",
                SettingKind::Num,
                "Tools",
            ),
            make(
                "Insecure Redirect",
                "HOMEBREW_NO_INSECURE_REDIRECT",
                "Forbid redirections from HTTPS to HTTP",
                SettingKind::Bool { inverted: true },
                "Security",
            ),
            make(
                "Bottle Domain",
                "HOMEBREW_BOTTLE_DOMAIN",
                "Custom mirror URL for bottle downloads",
                SettingKind::Str,
                "Network",
            ),
            make(
                "Brew Remote",
                "HOMEBREW_BREW_GIT_REMOTE",
                "Custom git remote URL for Homebrew/brew",
                SettingKind::Str,
                "Network",
            ),
            make(
                "Core Remote",
                "HOMEBREW_CORE_GIT_REMOTE",
                "Custom git remote URL for homebrew-core",
                SettingKind::Str,
                "Network",
            ),
            make(
                "GitHub Token",
                "HOMEBREW_GITHUB_API_TOKEN",
                "Personal access token for GitHub API (avoids rate limiting)",
                SettingKind::Str,
                "Network",
            ),
        ]
    }

    fn detect_shell_profile() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let shell = env::var("SHELL").unwrap_or_default();
        if shell.contains("zsh") {
            home.join(".zprofile")
        } else if shell.contains("fish") {
            home.join(".config").join("fish").join("config.fish")
        } else {
            home.join(".bash_profile")
        }
    }

    fn compute_list_index(settings: &[Setting], setting_idx: usize) -> usize {
        let mut list_index = 0;
        let mut current_category: Option<&str> = None;
        for (i, setting) in settings.iter().enumerate() {
            if current_category != Some(setting.category) {
                if current_category.is_some() {
                    list_index += 1;
                }
                list_index += 1;
                current_category = Some(setting.category);
            }
            if i == setting_idx {
                return list_index;
            }
            list_index += 1;
        }
        list_index.saturating_sub(1)
    }

    fn sync_list_state(&mut self) {
        let idx = Self::compute_list_index(&self.settings, self.selected);
        self.list_state.select(Some(idx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(kind: SettingKind) -> Setting {
        Setting {
            name: "Test",
            env_var: "HOMEBREW_TEST",
            description: "test",
            kind,
            bool_val: false,
            str_val: String::new(),
            num_val: None,
            category: "Test",
            modified: true,
        }
    }

    #[test]
    fn inverted_bool_is_on_when_var_unset() {
        let mut s = setting(SettingKind::Bool { inverted: true });
        s.apply_env_value(None);
        assert!(s.bool_val);
        s.apply_env_value(Some("1".to_string()));
        assert!(!s.bool_val);
    }

    #[test]
    fn non_inverted_bool_is_on_when_var_set() {
        let mut s = setting(SettingKind::Bool { inverted: false });
        s.apply_env_value(None);
        assert!(!s.bool_val);
        // Even an empty string means the variable is present.
        s.apply_env_value(Some(String::new()));
        assert!(s.bool_val);
    }

    #[test]
    fn str_reads_value_or_defaults_to_empty() {
        let mut s = setting(SettingKind::Str);
        s.apply_env_value(None);
        assert_eq!(s.str_val, "");
        s.apply_env_value(Some("/tmp/cache".to_string()));
        assert_eq!(s.str_val, "/tmp/cache");
    }

    #[test]
    fn num_parses_trimmed_value_and_rejects_garbage() {
        let mut s = setting(SettingKind::Num);
        s.apply_env_value(Some("  30 ".to_string()));
        assert_eq!(s.num_val, Some(30));
        s.apply_env_value(Some("abc".to_string()));
        assert_eq!(s.num_val, None);
        s.apply_env_value(None);
        assert_eq!(s.num_val, None);
    }

    #[test]
    fn apply_env_value_clears_modified_flag() {
        let mut s = setting(SettingKind::Str);
        assert!(s.modified);
        s.apply_env_value(None);
        assert!(!s.modified);
    }
}
