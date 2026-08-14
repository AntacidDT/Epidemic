// Epidemic NS — UI Theme
// Warm color palette + gradient helpers

use egui::Color32;

// ─── Color Palette ───
pub const PRIMARY: Color32 = Color32::from_rgb(255, 0, 0);       // #ff0000
pub const SECONDARY: Color32 = Color32::from_rgb(185, 41, 38);   // #b92926
pub const TERTIARY: Color32 = Color32::from_rgb(255, 87, 87);    // #ff5757
pub const BLACK: Color32 = Color32::from_rgb(7, 7, 7);           // #070707
pub const WHITE: Color32 = Color32::from_rgb(255, 255, 255);     // #ffffff
pub const EXTRA: Color32 = Color32::from_rgb(255, 161, 83);      // #ffa153

// Derived colors
pub const BG_DARK: Color32 = Color32::from_rgb(12, 12, 14);
pub const BG_PANEL: Color32 = Color32::from_rgb(18, 18, 22);
pub const BG_CARD: Color32 = Color32::from_rgb(28, 28, 34);
pub const BG_HOVER: Color32 = Color32::from_rgb(38, 38, 46);
pub const BORDER: Color32 = Color32::from_rgb(55, 55, 65);
pub const TEXT: Color32 = Color32::from_rgb(230, 230, 235);
pub const TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 155);
pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
pub const INFO: Color32 = Color32::from_rgb(59, 130, 246);

/// Apply the warm theme to egui context
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(16);

    // Colors
    style.visuals.window_fill = BG_PANEL;
    style.visuals.panel_fill = BG_DARK;
    style.visuals.override_text_color = Some(TEXT);

    // Widgets
    style.visuals.widgets.noninteractive.bg_fill = BG_CARD;
    style.visuals.widgets.inactive.bg_fill = BG_CARD;
    style.visuals.widgets.hovered.bg_fill = BG_HOVER;
    style.visuals.widgets.active.bg_fill = PRIMARY;

    style.visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, TERTIARY);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, PRIMARY);

    ctx.set_style(style);
}

/// Draw a horizontal gradient rectangle
pub fn gradient_rect(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    color_left: Color32,
    color_right: Color32,
) {
    let painter = ui.painter();
    let steps = 32;
    let step_w = rect.width() / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let r = lerp(color_left.r() as f32, color_right.r() as f32, t) as u8;
        let g = lerp(color_left.g() as f32, color_right.g() as f32, t) as u8;
        let b = lerp(color_left.b() as f32, color_right.b() as f32, t) as u8;
        let color = Color32::from_rgb(r, g, b);

        let x = rect.left() + step_w * i as f32;
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(x, rect.top()),
                egui::vec2(step_w + 1.0, rect.height()),
            ),
            0.0,
            color,
        );
    }
}

/// Draw a vertical gradient rectangle
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
        let color = Color32::from_rgb(r, g, b);

        let y = rect.top() + step_h * i as f32;
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left(), y),
                egui::vec2(rect.width(), step_h + 1.0),
            ),
            0.0,
            color,
        );
    }
}

/// Draw a gradient button
pub fn gradient_button(
    ui: &mut egui::Ui,
    label: &str,
    size: f32,
    color_left: Color32,
    color_right: Color32,
) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(size).strong().color(WHITE),
    )
    .min_size(egui::vec2(180.0, 48.0))
    .fill(color_left) // egui doesn't support gradient fills, use primary color
    .corner_radius(egui::CornerRadius::same(10));

    ui.add(btn).clicked()
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Format large numbers
pub fn fmt_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

/// Stat row helper
pub fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(13.0).color(TEXT_DIM));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).size(13.0).strong().color(color));
        });
    });
}
