// Epidemic NS — Design System (9C44N0)
// All UI components follow these exact specs.

use egui::{Color32, Pos2, Rect, Stroke, Ui, FontId};

// ─── Color Constants ───
pub const CORAL_RED: Color32 = Color32::from_rgb(255, 87, 87);      // #ff5757
pub const RED_STROKE: Color32 = Color32::from_rgb(185, 41, 38);     // #b92926
pub const TITLE_COLOR: Color32 = Color32::from_rgb(255, 85, 119);   // #ff5577
pub const TITLE_STROKE: Color32 = Color32::from_rgb(185, 41, 38);   // #b92926
pub const CURE_BLUE: Color32 = Color32::from_rgb(95, 175, 239);     // #5fafef
pub const CURE_STROKE: Color32 = Color32::from_rgb(86, 137, 231);   // #5689e7
pub const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
pub const WHITE: Color32 = Color32::from_rgb(255, 255, 255);

// ─── Module: Belowbrick ───
// White panel, 45% transparent, black stroke
pub struct BelowbrickStyle {
    pub fill: Color32,
    pub stroke_color: Color32,
    pub stroke_width: f32,
    pub corner_radius: f32,
}

impl Default for BelowbrickStyle {
    fn default() -> Self {
        Self {
            fill: Color32::from_rgba_unmultiplied(255, 255, 255, 115),
            stroke_color: BLACK,
            stroke_width: 1.0,
            corner_radius: 0.0,
        }
    }
}

pub fn draw_belowbrick(ui: &mut Ui, rect: Rect) {
    let style = BelowbrickStyle::default();
    ui.painter().rect_filled(rect, style.corner_radius, style.fill);
    ui.painter().rect_stroke(rect, style.corner_radius, Stroke::new(style.stroke_width, style.stroke_color), egui::StrokeKind::Outside);
}

// ─── Module: Text Button ───
// Coral red, opaque, red stroke, grows on hover, black text
pub fn draw_text_button(ui: &mut Ui, rect: Rect, label: &str, is_hovered: bool) -> bool {
    let scale = if is_hovered { 1.08 } else { 1.0 };
    let w = rect.width() * scale;
    let h = rect.height() * scale;
    let x = rect.center().x - w * 0.5;
    let y = rect.center().y - h * 0.5;
    let scaled_rect = Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h));

    // Fill
    ui.painter().rect_filled(scaled_rect, 6.0, CORAL_RED);
    // Stroke
    ui.painter().rect_stroke(scaled_rect, 6.0, Stroke::new(2.0, RED_STROKE), egui::StrokeKind::Outside);
    // Text
    let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::proportional(14.0), BLACK);
    let text_pos = Pos2::new(
        scaled_rect.center().x - galley.size().x * 0.5,
        scaled_rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, BLACK);

    // Click detection on original rect
    let response = ui.allocate_rect(rect, egui::Sense::click());
    response.clicked()
}

// ─── Module: Button ───
// Coral red, opaque, red stroke, grows on hover
pub fn draw_button(ui: &mut Ui, rect: Rect, is_hovered: bool) -> bool {
    let scale = if is_hovered { 1.08 } else { 1.0 };
    let w = rect.width() * scale;
    let h = rect.height() * scale;
    let x = rect.center().x - w * 0.5;
    let y = rect.center().y - h * 0.5;
    let scaled_rect = Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h));

    ui.painter().rect_filled(scaled_rect, 6.0, CORAL_RED);
    ui.painter().rect_stroke(scaled_rect, 6.0, Stroke::new(2.0, RED_STROKE), egui::StrokeKind::Outside);

    let response = ui.allocate_rect(rect, egui::Sense::click());
    response.clicked()
}

// ─── Module: Title ───
// Coral red (#ff5577), red stroke (#b92926), 130px, centered
pub fn draw_title(ui: &mut Ui, center: Pos2, text: &str) {
    draw_outlined_text_module(ui, text, center, 130.0, TITLE_COLOR, TITLE_STROKE, true);
}

// ─── Module: Subtitle ───
// Coral red, red stroke, 50.9px, centered
pub fn draw_subtitle(ui: &mut Ui, center: Pos2, text: &str) {
    draw_outlined_text_module(ui, text, center, 50.9, TITLE_COLOR, TITLE_STROKE, true);
}

// ─── Module: Text ───
// Coral red, red stroke, 22.5px default (max 30), centered
pub fn draw_text(ui: &mut Ui, center: Pos2, text: &str, size: f32) {
    let size = size.clamp(10.0, 30.0);
    draw_outlined_text_module(ui, text, center, size, TITLE_COLOR, TITLE_STROKE, true);
}

// ─── Module: Subtext ───
// Coral red, red stroke, 14px default (max 20), centered
pub fn draw_subtext(ui: &mut Ui, center: Pos2, text: &str, size: f32) {
    let size = size.clamp(8.0, 20.0);
    draw_outlined_text_module(ui, text, center, size, TITLE_COLOR, TITLE_STROKE, true);
}

// ─── Module: Text Field ───
// Coral red, red stroke, 22.5px default (max 30), centered
pub fn draw_text_field(ui: &mut Ui, rect: Rect, text: &str, size: f32) {
    let size = size.clamp(10.0, 30.0);
    // Background
    ui.painter().rect_filled(rect, 4.0, CORAL_RED);
    ui.painter().rect_stroke(rect, 4.0, Stroke::new(2.0, RED_STROKE), egui::StrokeKind::Outside);
    // Text centered
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(size), BLACK);
    let text_pos = Pos2::new(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, BLACK);
}

// ─── Cure Mode Colors ───
// If text is about Cure mode, use vivid azure (#5fafef) with blue stroke (#5689e7)
pub fn draw_cure_text(ui: &mut Ui, center: Pos2, text: &str, size: f32) {
    let size = size.clamp(10.0, 30.0);
    draw_outlined_text_module(ui, text, center, size, CURE_BLUE, CURE_STROKE, true);
}

// ─── Core outlined text renderer ───
fn draw_outlined_text_module(
    ui: &mut Ui,
    text: &str,
    center: Pos2,
    size: f32,
    fill: Color32,
    outline: Color32,
    centered: bool,
) {
    let font = FontId::proportional(size);

    // Outline in 4 directions
    for (ox, oy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), outline);
        let pos = if centered {
            Pos2::new(center.x - galley.size().x * 0.5 + ox, center.y - galley.size().y * 0.5 + oy)
        } else {
            Pos2::new(center.x + ox, center.y + oy)
        };
        ui.painter().galley(pos, galley, outline);
    }

    // Fill
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, fill);
    let pos = if centered {
        Pos2::new(center.x - galley.size().x * 0.5, center.y - galley.size().y * 0.5)
    } else {
        center
    };
    ui.painter().galley(pos, galley, fill);
}
