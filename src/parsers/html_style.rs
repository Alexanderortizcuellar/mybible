use regex::Regex;
use serde::{Deserialize, Serialize};

/// Theme color definitions according to MyBible specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub text: String,
    pub red: String,
    pub green: String,
    pub blue: String,
    pub purple: String,
    pub grey: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::day()
    }
}

impl ThemeColors {
    pub fn day() -> Self {
        Self {
            text: "#000000".to_string(),
            red: "#cc0000".to_string(),
            green: "#008800".to_string(),
            blue: "#0000cc".to_string(),
            purple: "#880088".to_string(),
            grey: "#666666".to_string(),
        }
    }

    pub fn night() -> Self {
        Self {
            text: "#e0e0e0".to_string(),
            red: "#ff6666".to_string(),
            green: "#66cc66".to_string(),
            blue: "#66aaff".to_string(),
            purple: "#cc88cc".to_string(),
            grey: "#999999".to_string(),
        }
    }

    pub fn color_by_index(&self, index: u8) -> &str {
        match index {
            0 => &self.text,
            1 => &self.red,
            2 => &self.green,
            3 => &self.blue,
            4 => &self.purple,
            5 => &self.grey,
            _ => &self.text,
        }
    }
}

lazy_static::lazy_static! {
    static ref FONT_INDEX_REGEX: Regex = Regex::new(
        r#"(?i)<font\s+([^>]*?)color\s*=\s*['"]?([0-5])['"]?([^>]*?)>"#
    ).unwrap();
    static ref FONT_KEYWORD_REGEX: Regex = Regex::new(
        r#"(?i)<font\s+([^>]*?)color\s*=\s*['"]?\s*(%COLOR_[A-Z]+%)\s*['"]?([^>]*?)>"#
    ).unwrap();
}

/// Applies theme colors and replaces MyBible color placeholders in HTML.
pub fn apply_theme_colors(html: &str, theme: &ThemeColors) -> String {
    // 1. Replace %COLOR_*% keywords
    let mut out = html
        .replace("%COLOR_TEXT%", &theme.text)
        .replace("%COLOR_RED%", &theme.red)
        .replace("%COLOR_GREEN%", &theme.green)
        .replace("%COLOR_BLUE%", &theme.blue)
        .replace("%COLOR_PURPLE%", &theme.purple)
        .replace("%COLOR_GREY%", &theme.grey);

    // 2. Replace indexed <font color='0..5'> with actual color hex
    out = FONT_INDEX_REGEX
        .replace_all(&out, |caps: &regex::Captures| {
            let before = &caps[1];
            let idx: u8 = caps[2].parse().unwrap_or(0);
            let after = &caps[3];
            let color = theme.color_by_index(idx);
            format!("<font {}color=\"{}\"{}>", before, color, after)
        })
        .to_string();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_replacement() {
        let theme = ThemeColors::day();
        let html = "<font color='1'>Red text</font> and %COLOR_BLUE% text";
        let res = apply_theme_colors(html, &theme);
        assert!(res.contains(&format!("color=\"{}\"", theme.red)));
        assert!(res.contains(&theme.blue));
    }
}
