use eframe::egui::{self, Color32, RichText, Sense, Ui, Vec2};
// `RichText` is used in section_header below.

/// A sidebar tab button that lights up when active.
pub fn sidebar_button(ui: &mut Ui, label: &str, icon: &str, active: bool, badge: Option<String>) -> bool {
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
    let font = egui::FontId::proportional(if active { 14.0 } else { 14.0 });
    ui.painter().text(
        rect.left_center() + Vec2::new(14.0, 0.0),
        egui::Align2::LEFT_CENTER,
        format!("{icon}  {label}"),
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
