use ratatui::style::Color;

/// A colour palette for the whole UI. All UI colours are sourced from the
/// active theme so the look can be swapped at runtime.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub primary: Color,     // titles, selected pointer
    pub accent: Color,      // borders, values
    pub on: Color,          // enabled / on
    pub off: Color,         // disabled / dim labels
    pub modified: Color,    // modified marker, profile path
    pub category: Color,    // category headers
    pub bg: Color,          // background
    pub selected_bg: Color, // selected row background
    pub error: Color,       // error messages
    pub warning: Color,     // warnings (e.g. missing path)
    pub text: Color,        // primary text
    pub text_dim: Color,    // secondary text
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// All available themes. The first one is the default.
pub const THEMES: &[Theme] = &[
    Theme {
        name: "brew",
        primary: rgb(255, 180, 0),
        accent: rgb(230, 120, 0),
        on: rgb(80, 200, 120),
        off: rgb(120, 120, 130),
        modified: rgb(100, 180, 255),
        category: rgb(180, 140, 255),
        bg: rgb(22, 22, 32),
        selected_bg: rgb(40, 40, 55),
        error: rgb(230, 90, 90),
        warning: rgb(230, 160, 60),
        text: rgb(235, 235, 240),
        text_dim: rgb(150, 150, 160),
    },
    Theme {
        name: "midnight",
        primary: rgb(120, 180, 255),
        accent: rgb(90, 140, 230),
        on: rgb(90, 210, 160),
        off: rgb(100, 110, 130),
        modified: rgb(180, 150, 255),
        category: rgb(130, 200, 255),
        bg: rgb(16, 18, 28),
        selected_bg: rgb(32, 38, 58),
        error: rgb(235, 100, 110),
        warning: rgb(235, 180, 90),
        text: rgb(230, 235, 245),
        text_dim: rgb(150, 160, 180),
    },
    Theme {
        name: "forest",
        primary: rgb(150, 220, 120),
        accent: rgb(90, 170, 90),
        on: rgb(120, 220, 140),
        off: rgb(110, 125, 110),
        modified: rgb(205, 200, 120),
        category: rgb(120, 200, 180),
        bg: rgb(18, 24, 20),
        selected_bg: rgb(32, 46, 36),
        error: rgb(230, 120, 100),
        warning: rgb(220, 190, 100),
        text: rgb(225, 235, 220),
        text_dim: rgb(150, 170, 150),
    },
    Theme {
        name: "rose",
        primary: rgb(255, 150, 200),
        accent: rgb(230, 110, 160),
        on: rgb(150, 210, 170),
        off: rgb(160, 125, 145),
        modified: rgb(200, 160, 255),
        category: rgb(240, 160, 230),
        bg: rgb(28, 18, 26),
        selected_bg: rgb(52, 34, 48),
        error: rgb(240, 110, 120),
        warning: rgb(240, 180, 120),
        text: rgb(245, 225, 240),
        text_dim: rgb(195, 165, 185),
    },
    Theme {
        name: "mono",
        primary: rgb(235, 235, 235),
        accent: rgb(175, 175, 175),
        on: rgb(205, 205, 205),
        off: rgb(110, 110, 110),
        modified: rgb(160, 160, 160),
        category: rgb(200, 200, 200),
        bg: rgb(20, 20, 20),
        selected_bg: rgb(44, 44, 44),
        error: rgb(220, 120, 120),
        warning: rgb(220, 190, 120),
        text: rgb(245, 245, 245),
        text_dim: rgb(150, 150, 150),
    },
];

/// Index of the theme with the given name, if any.
pub fn index_of(name: &str) -> Option<usize> {
    THEMES.iter().position(|t| t.name == name)
}

/// Comma-separated list of theme names, for help text and error messages.
pub fn names() -> String {
    THEMES.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_brew() {
        assert_eq!(THEMES[0].name, "brew");
    }

    #[test]
    fn theme_names_are_unique() {
        for (i, t) in THEMES.iter().enumerate() {
            assert_eq!(index_of(t.name), Some(i));
        }
    }

    #[test]
    fn index_of_unknown_is_none() {
        assert_eq!(index_of("nope"), None);
    }

    #[test]
    fn names_lists_every_theme() {
        let listed = names();
        for t in THEMES {
            assert!(listed.contains(t.name));
        }
    }
}
