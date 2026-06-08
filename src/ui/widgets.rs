use eframe::egui::{self, Color32, Rect, RichText, Sense, Ui, Vec2};

use super::tab_icons;

/// A sidebar tab button that lights up when active.
pub fn sidebar_button(ui: &mut Ui, label: &str, icon_key: &str, active: bool, badge: Option<String>) -> bool {
    let desired = Vec2::new(ui.available_width(), 38.0);
    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());
    let visuals = &ui.style().visuals;
    let bg = if active {
        visuals.selection.bg_fill
    } else if resp.hovered() {
        visuals.widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(6), bg);
    let text_color = if active { visuals.strong_text_color() } else { visuals.text_color() };
    let badge_color = visuals.hyperlink_color;
    let font = egui::FontId::proportional(14.0);

    // Icon: 18x18 box, vertically centered, 12px from the left edge
    let icon_size = 18.0_f32;
    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.left() + 12.0 + icon_size * 0.5, rect.center().y),
        Vec2::new(icon_size, icon_size),
    );
    tab_icons::draw_in_rect(ui.painter(), icon_key, icon_rect, text_color);

    // Text starts after the icon with a small gap
    let text_x = icon_rect.right() + 8.0;
    ui.painter().text(
        egui::pos2(text_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );

    if let Some(b) = badge {
        let w = b.chars().count() as f32 * 7.0 + 16.0;
        let badge_rect = egui::Rect::from_min_size(
            rect.right_center() + Vec2::new(-(w + 8.0), -10.0),
            Vec2::new(w, 20.0),
        );
        ui.painter().rect_filled(badge_rect, egui::CornerRadius::same(10), badge_color);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            b,
            egui::FontId::proportional(12.0),
            visuals.strong_text_color(),
        );
    }
    resp.clicked()
}

/// A small colored pill (used for category labels).
pub fn pill(ui: &mut Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(48.0, 18.0), Sense::hover());
    ui.painter().rect_filled(rect, egui::CornerRadius::same(10), color);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
}

/// A small heading row with optional subtitle.
pub fn section_header(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.add_space(4.0);
    ui.label(RichText::new(title).size(18.0).strong());
    if let Some(s) = subtitle {
        ui.label(RichText::new(s).size(12.0).weak());
    }
    ui.add_space(4.0);
    ui.separator();
}
