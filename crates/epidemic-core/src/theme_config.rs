use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub colors: ColorsConfig,
    pub fonts: FontsConfig,
    pub splash: SplashConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorsConfig {
    pub primary: [u8; 3],
    pub secondary: [u8; 3],
    pub tertiary: [u8; 3],
    pub extra: [u8; 3],
    pub bg_dark: [u8; 3],
    pub bg_panel: [u8; 3],
    pub bg_card: [u8; 3],
    pub bg_hover: [u8; 3],
    pub text: [u8; 3],
    pub text_dim: [u8; 3],
    pub heading: [u8; 3],
    pub border: [u8; 3],
    pub success: [u8; 3],
    pub danger: [u8; 3],
    pub warning: [u8; 3],
    pub info: [u8; 3],
    pub ocean: [u8; 3],
    pub healthy: [u8; 3],
    pub infected: [u8; 3],
    pub dead: [u8; 3],
    pub coral_red: [u8; 3],
    pub sky_blue: [u8; 3],
    pub lavender: [u8; 3],
    pub dark_maroon: [u8; 3],
    pub panel_fill: [u8; 3],
    pub btn_fill: [u8; 3],
    pub btn_outline: [u8; 3],
    pub btn_text: [u8; 3],
    pub title_color: [u8; 3],
}

#[derive(Debug, Clone, Deserialize)]
pub struct FontsConfig {
    pub title_size: f32,
    pub subtitle_size: f32,
    pub heading_size: f32,
    pub body_size: f32,
    pub small_size: f32,
    pub tiny_size: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SplashConfig {
    pub duration_ms: u64,
    pub fade_out_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiConfig {
    pub sidebar_width: f32,
    pub button_height: f32,
    pub card_radius: u8,
    pub button_radius: u8,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            colors: ColorsConfig {
                primary: [255, 0, 0],
                secondary: [185, 41, 38],
                tertiary: [255, 87, 87],
                extra: [255, 161, 83],
                bg_dark: [12, 12, 14],
                bg_panel: [18, 18, 22],
                bg_card: [28, 28, 34],
                bg_hover: [38, 38, 46],
                text: [230, 230, 235],
                text_dim: [140, 140, 155],
                heading: [255, 255, 255],
                border: [55, 55, 65],
                success: [34, 197, 94],
                danger: [239, 68, 68],
                warning: [245, 158, 11],
                info: [59, 130, 246],
                ocean: [5, 10, 25],
                healthy: [20, 70, 30],
                infected: [200, 50, 30],
                dead: [60, 50, 50],
                coral_red: [255, 102, 102],
                sky_blue: [51, 153, 255],
                lavender: [153, 170, 204],
                dark_maroon: [58, 11, 14],
                panel_fill: [232, 130, 130],
                btn_fill: [255, 92, 92],
                btn_outline: [58, 11, 14],
                btn_text: [0, 0, 0],
                title_color: [128, 24, 24],
            },
            fonts: FontsConfig {
                title_size: 72.0,
                subtitle_size: 28.0,
                heading_size: 20.0,
                body_size: 14.0,
                small_size: 12.0,
                tiny_size: 10.0,
            },
            splash: SplashConfig {
                duration_ms: 2000,
                fade_out_ms: 500,
            },
            ui: UiConfig {
                sidebar_width: 0.28,
                button_height: 48.0,
                card_radius: 8,
                button_radius: 6,
            },
        }
    }
}

impl ThemeConfig {
    pub fn load() -> Self {
        let paths = [
            "../Assets/theme.toml",
            "Assets/theme.toml",
            "../assets/theme.toml",
            "assets/theme.toml",
        ];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(config) = toml::from_str::<ThemeConfig>(&content) {
                    println!("Loaded theme from {path}");
                    return config;
                }
            }
        }

        println!("No theme.toml found, using defaults");
        Self::default()
    }
}
