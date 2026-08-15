// Epidemic NS — Theme System
// Loads colors/fonts from theme.toml at runtime.
// Edit Assets/theme.toml to customize without recompiling.

use egui::Color32;
use epidemic_core::ThemeConfig;

// ─── Runtime-loaded colors (initialized from theme.toml) ───
use std::sync::OnceLock;
static THEME: OnceLock<ThemeConfig> = OnceLock::new();

pub fn init_theme() {
    THEME.get_or_init(|| ThemeConfig::load());
}

fn t() -> &'static ThemeConfig {
    THEME.get().expect("Theme not initialized")
}

// ─── Color accessors ───
pub fn primary() -> Color32 { let c = t().colors.primary; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn secondary() -> Color32 { let c = t().colors.secondary; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn tertiary() -> Color32 { let c = t().colors.tertiary; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn extra() -> Color32 { let c = t().colors.extra; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn bg_dark() -> Color32 { let c = t().colors.bg_dark; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn bg_panel() -> Color32 { let c = t().colors.bg_panel; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn bg_card() -> Color32 { let c = t().colors.bg_card; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn bg_hover() -> Color32 { let c = t().colors.bg_hover; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn text_color() -> Color32 { let c = t().colors.text; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn text_dim() -> Color32 { let c = t().colors.text_dim; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn heading_color() -> Color32 { let c = t().colors.heading; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn border_color() -> Color32 { let c = t().colors.border; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn success_color() -> Color32 { let c = t().colors.success; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn danger_color() -> Color32 { let c = t().colors.danger; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn warning_color() -> Color32 { let c = t().colors.warning; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn info_color() -> Color32 { let c = t().colors.info; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn ocean_color() -> Color32 { let c = t().colors.ocean; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn healthy_color() -> Color32 { let c = t().colors.healthy; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn infected_color() -> Color32 { let c = t().colors.infected; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn dead_color() -> Color32 { let c = t().colors.dead; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn coral_red() -> Color32 { let c = t().colors.coral_red; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn sky_blue() -> Color32 { let c = t().colors.sky_blue; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn lavender() -> Color32 { let c = t().colors.lavender; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn dark_maroon() -> Color32 { let c = t().colors.dark_maroon; Color32::from_rgb(c[0], c[1], c[2]) }

pub fn panel_fill() -> Color32 { let c = t().colors.panel_fill; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn btn_fill() -> Color32 { let c = t().colors.btn_fill; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn btn_outline() -> Color32 { let c = t().colors.btn_outline; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn btn_text_color() -> Color32 { let c = t().colors.btn_text; Color32::from_rgb(c[0], c[1], c[2]) }
pub fn title_color() -> Color32 { let c = t().colors.title_color; Color32::from_rgb(c[0], c[1], c[2]) }

// ─── Font sizes ───
pub fn title_size() -> f32 { t().fonts.title_size }
pub fn subtitle_size() -> f32 { t().fonts.subtitle_size }
pub fn heading_size() -> f32 { t().fonts.heading_size }
pub fn body_size() -> f32 { t().fonts.body_size }
pub fn small_size() -> f32 { t().fonts.small_size }
pub fn tiny_size() -> f32 { t().fonts.tiny_size }

// ─── Splash ───
pub fn splash_duration_ms() -> u64 { t().splash.duration_ms }
pub fn splash_fade_ms() -> u64 { t().splash.fade_out_ms }

// ─── UI ───
pub fn sidebar_width() -> f32 { t().ui.sidebar_width }
pub fn button_height() -> f32 { t().ui.button_height }
pub fn card_radius() -> u8 { t().ui.card_radius }
pub fn button_radius() -> u8 { t().ui.button_radius }

// ─── Apply theme to egui ───
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.visuals.window_fill = bg_panel();
    style.visuals.panel_fill = bg_dark();
    style.visuals.override_text_color = Some(text_color());
    style.visuals.widgets.noninteractive.bg_fill = bg_card();
    style.visuals.widgets.inactive.bg_fill = bg_card();
    style.visuals.widgets.hovered.bg_fill = bg_hover();
    style.visuals.widgets.active.bg_fill = primary();
    style.visuals.window_stroke = egui::Stroke::new(1.0, border_color());
    ctx.set_style(style);
}

// ─── Gradient helpers ───
pub fn gradient_rect_vertical(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    color_top: Color32,
    color_bottom: Color32,
) {
    let painter = ui.painter();
    let steps = 32;
    let step_h = rect.height() / steps as f32;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let r = lerp(color_top.r() as f32, color_bottom.r() as f32, t) as u8;
        let g = lerp(color_top.g() as f32, color_bottom.g() as f32, t) as u8;
        let b = lerp(color_top.b() as f32, color_bottom.b() as f32, t) as u8;
        let y = rect.top() + step_h * i as f32;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(rect.left(), y), egui::vec2(rect.width(), step_h + 1.0)),
            0.0,
            Color32::from_rgb(r, g, b),
        );
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

// ─── Text with outline ───
pub fn draw_outlined_text(
    ui: &mut egui::Ui,
    text: &str,
    center_pos: egui::Pos2,
    font_id: egui::FontId,
    fill_color: Color32,
    outline_color: Color32,
    centered: bool,
) {
    let offsets = [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)];
    for (ox, oy) in offsets {
        let galley = ui.painter().layout_no_wrap(text.to_string(), font_id.clone(), outline_color);
        let pos = if centered {
            egui::pos2(center_pos.x - galley.size().x * 0.5 + ox, center_pos.y - galley.size().y * 0.5 + oy)
        } else {
            egui::pos2(center_pos.x + ox, center_pos.y + oy)
        };
        ui.painter().galley(pos, galley, outline_color);
    }
    let galley = ui.painter().layout_no_wrap(text.to_string(), font_id, fill_color);
    let pos = if centered {
        egui::pos2(center_pos.x - galley.size().x * 0.5, center_pos.y - galley.size().y * 0.5)
    } else {
        center_pos
    };
    ui.painter().galley(pos, galley, fill_color);
}

// ─── Number formatting ───
pub fn fmt_num(n: u64) -> String {
    if n >= 1_000_000_000 { format!("{:.1}B", n as f64 / 1_000_000_000.0) }
    else if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { format!("{n}") }
}
