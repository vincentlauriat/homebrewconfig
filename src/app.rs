use std::env;
use std::path::{Path, PathBuf};

use ratatui::widgets::ListState;

use crate::config;

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
    /// Human-readable Homebrew default, shown in the detail pane. Empty when
    /// there is no meaningful documented default.
    pub default_hint: &'static str,
}

impl Setting {
    fn with_default(mut self, hint: &'static str) -> Self {
        self.default_hint = hint;
        self
    }
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
    Filtering,
}

pub struct App {
    pub settings: Vec<Setting>,
    pub selected: usize,
    pub list_state: ListState,
    pub mode: Mode,
    pub input: String,
    pub cursor_pos: usize,
    pub filter: String,
    pub shell_profile: PathBuf,
    pub profile_candidates: Vec<PathBuf>,
    pub message: Option<(String, bool)>,
    pub show_help: bool,
}

impl App {
    // Convenience constructor used by tests; the binary entry point goes
    // through `with_profile` to honour the `--profile` flag.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_profile(None)
    }

    /// Build the app, optionally forcing a specific profile path (e.g. from a
    /// `--profile` CLI flag). When `None`, the target is auto-detected.
    pub fn with_profile(override_path: Option<PathBuf>) -> Self {
        let mut settings = Self::build_settings();
        for setting in &mut settings {
            setting.read_from_env();
        }
        let candidates = Self::profile_candidates();
        let shell_profile = override_path.unwrap_or_else(|| {
            Self::pick_profile(&candidates, config::file_contains_block, |p| p.exists())
        });
        let mut app = App {
            settings,
            selected: 0,
            list_state: ListState::default(),
            mode: Mode::Normal,
            input: String::new(),
            cursor_pos: 0,
            filter: String::new(),
            shell_profile,
            profile_candidates: candidates,
            message: None,
            show_help: false,
        };
        app.sync_list_state();
        app
    }

    /// Cycle the write target through the detected candidate profiles.
    pub fn cycle_profile(&mut self) {
        if self.profile_candidates.is_empty() {
            return;
        }
        let pos = self
            .profile_candidates
            .iter()
            .position(|p| p == &self.shell_profile);
        let next = match pos {
            Some(i) => (i + 1) % self.profile_candidates.len(),
            None => 0,
        };
        self.shell_profile = self.profile_candidates[next].clone();
        let disp = self.shell_profile.display().to_string();
        self.message = Some((format!("Target profile: {}", disp), false));
    }

    /// Indices into `settings` that match the current filter, in order.
    /// An empty filter shows everything.
    pub fn visible(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            return (0..self.settings.len()).collect();
        }
        let needle = self.filter.to_lowercase();
        self.settings
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.name.to_lowercase().contains(&needle)
                    || s.env_var.to_lowercase().contains(&needle)
                    || s.category.to_lowercase().contains(&needle)
                    || s.description.to_lowercase().contains(&needle)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn start_filter(&mut self) {
        self.mode = Mode::Filtering;
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.ensure_selected_visible();
    }

    pub fn filter_backspace(&mut self) {
        self.filter.pop();
        self.ensure_selected_visible();
    }

    /// Keep the filter but leave filtering mode.
    pub fn confirm_filter(&mut self) {
        self.mode = Mode::Normal;
        self.ensure_selected_visible();
    }

    /// Drop the filter entirely and leave filtering mode.
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.mode = Mode::Normal;
        self.ensure_selected_visible();
    }

    fn ensure_selected_visible(&mut self) {
        let vis = self.visible();
        if !vis.is_empty() && !vis.contains(&self.selected) {
            self.selected = vis[0];
        }
        self.sync_list_state();
    }

    pub fn toggle_current(&mut self) {
        if !self.visible().contains(&self.selected) {
            return;
        }
        let setting = &mut self.settings[self.selected];
        if matches!(setting.kind, SettingKind::Bool { .. }) {
            setting.bool_val = !setting.bool_val;
            setting.modified = true;
        }
    }

    pub fn start_editing(&mut self) {
        if !self.visible().contains(&self.selected) {
            return;
        }
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
        let vis = self.visible();
        if vis.is_empty() {
            return;
        }
        let pos = vis.iter().position(|&i| i == self.selected).unwrap_or(0);
        self.selected = vis[(pos + 1) % vis.len()];
        self.sync_list_state();
    }

    pub fn select_prev(&mut self) {
        let vis = self.visible();
        if vis.is_empty() {
            return;
        }
        let pos = vis.iter().position(|&i| i == self.selected).unwrap_or(0);
        let prev = if pos == 0 { vis.len() - 1 } else { pos - 1 };
        self.selected = vis[prev];
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
            default_hint: "",
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
            )
            .with_default("120"),
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
            )
            .with_default("🍺"),
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
            )
            .with_default("$EDITOR / $VISUAL"),
            make(
                "Curl Retries",
                "HOMEBREW_CURL_RETRIES",
                "Number of times to retry failed downloads (default: 3)",
                SettingKind::Num,
                "Tools",
            )
            .with_default("3"),
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

    /// Candidate profile files for the current shell, in preference order.
    fn profile_candidates() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let shell = env::var("SHELL").unwrap_or_default();
        if shell.contains("zsh") {
            vec![home.join(".zshrc"), home.join(".zprofile")]
        } else if shell.contains("fish") {
            vec![home.join(".config").join("fish").join("config.fish")]
        } else {
            vec![
                home.join(".bashrc"),
                home.join(".bash_profile"),
                home.join(".profile"),
            ]
        }
    }

    /// Choose which candidate to write to:
    /// 1. one that already holds our managed block (cross-file idempotence),
    /// 2. otherwise the first that already exists,
    /// 3. otherwise the first (preferred) candidate.
    ///
    /// `has_block` and `exists` are injected so the policy is unit-testable
    /// without touching the filesystem.
    fn pick_profile(
        candidates: &[PathBuf],
        has_block: impl Fn(&Path) -> bool,
        exists: impl Fn(&Path) -> bool,
    ) -> PathBuf {
        if let Some(p) = candidates.iter().find(|p| has_block(p)) {
            return p.clone();
        }
        if let Some(p) = candidates.iter().find(|p| exists(p)) {
            return p.clone();
        }
        candidates
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".profile"))
    }

    /// Map `selected` to its visual row in the rendered list, accounting for
    /// the category headers and blank separators inserted between groups.
    fn sync_list_state(&mut self) {
        let vis = self.visible();
        let mut list_index = 0usize;
        let mut current_category: Option<&str> = None;
        for &i in &vis {
            let setting = &self.settings[i];
            if current_category != Some(setting.category) {
                if current_category.is_some() {
                    list_index += 1;
                }
                list_index += 1;
                current_category = Some(setting.category);
            }
            if i == self.selected {
                self.list_state.select(Some(list_index));
                return;
            }
            list_index += 1;
        }
        self.list_state.select(Some(0));
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
            default_hint: "",
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

    // ---- filtering ----

    #[test]
    fn empty_filter_shows_all_settings() {
        let app = App::new();
        assert_eq!(app.visible().len(), app.settings.len());
    }

    #[test]
    fn filter_narrows_to_matching_category() {
        let mut app = App::new();
        for c in "network".chars() {
            app.filter_push(c);
        }
        let vis = app.visible();
        assert!(!vis.is_empty());
        assert!(vis.iter().all(|&i| app.settings[i].category == "Network"));
    }

    #[test]
    fn filter_moves_selection_into_visible_set() {
        let mut app = App::new();
        app.selected = 0; // Analytics (Privacy) — not in the Network group
        for c in "network".chars() {
            app.filter_push(c);
        }
        assert!(app.visible().contains(&app.selected));
        assert_eq!(app.settings[app.selected].category, "Network");
    }

    #[test]
    fn select_next_stays_within_filtered_set() {
        let mut app = App::new();
        for c in "network".chars() {
            app.filter_push(c);
        }
        let vis = app.visible();
        app.select_next();
        assert!(vis.contains(&app.selected));
    }

    #[test]
    fn clear_filter_restores_full_list() {
        let mut app = App::new();
        for c in "network".chars() {
            app.filter_push(c);
        }
        app.clear_filter();
        assert!(app.filter.is_empty());
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.visible().len(), app.settings.len());
    }

    // ---- profile target selection ----

    fn zsh_candidates() -> Vec<PathBuf> {
        vec![PathBuf::from("/h/.zshrc"), PathBuf::from("/h/.zprofile")]
    }

    #[test]
    fn pick_profile_prefers_file_with_existing_block() {
        let c = zsh_candidates();
        // .zprofile holds our block even though both exist -> it wins.
        let chosen = App::pick_profile(&c, |p| p.ends_with(".zprofile"), |_| true);
        assert_eq!(chosen, PathBuf::from("/h/.zprofile"));
    }

    #[test]
    fn pick_profile_falls_back_to_first_existing() {
        let c = zsh_candidates();
        // No block anywhere; only .zprofile exists on disk.
        let chosen = App::pick_profile(&c, |_| false, |p| p.ends_with(".zprofile"));
        assert_eq!(chosen, PathBuf::from("/h/.zprofile"));
    }

    #[test]
    fn pick_profile_defaults_to_first_candidate() {
        let c = zsh_candidates();
        // Nothing exists and no block -> prefer the first candidate (.zshrc).
        let chosen = App::pick_profile(&c, |_| false, |_| false);
        assert_eq!(chosen, PathBuf::from("/h/.zshrc"));
    }

    #[test]
    fn cycle_profile_rotates_through_candidates() {
        let mut app = App::new();
        app.profile_candidates = zsh_candidates();
        app.shell_profile = PathBuf::from("/h/.zshrc");
        app.cycle_profile();
        assert_eq!(app.shell_profile, PathBuf::from("/h/.zprofile"));
        app.cycle_profile();
        assert_eq!(app.shell_profile, PathBuf::from("/h/.zshrc"));
    }
}
