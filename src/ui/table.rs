use eframe::egui::{self, RichText, TextEdit, Ui};

use crate::model::WorldInfoEntry;

/// Editable table view of all entries.
///
/// Columns: enabled (checkbox), UID, Name, Keys (comma-joined), Priority,
/// Position, Selective, Constant, Content preview. All visible cells are
/// inline-editable. Double-clicking a row opens a content editor window.
pub fn draw(ui: &mut Ui, entries: &mut Vec<WorldInfoEntry>, selected: &mut Option<u64>) {
    let mut to_remove: Vec<u64> = Vec::new();
    let mut content_editor: Option<(u64, String)> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Header row
            ui.horizontal(|ui| {
                ui.add_sized([60.0, 22.0], egui::Label::new(RichText::new("On").strong().size(12.0)));
                ui.add_sized([44.0, 22.0], egui::Label::new(RichText::new("UID").strong().size(12.0)));
                ui.add_sized([160.0, 22.0], egui::Label::new(RichText::new("Name").strong().size(12.0)));
                ui.add_sized([240.0, 22.0], egui::Label::new(RichText::new("Keys").strong().size(12.0)));
                ui.add_sized([70.0, 22.0], egui::Label::new(RichText::new("Priority").strong().size(12.0)));
                ui.add_sized([60.0, 22.0], egui::Label::new(RichText::new("Position").strong().size(12.0)));
                ui.add_sized([60.0, 22.0], egui::Label::new(RichText::new("Selective").strong().size(12.0)));
                ui.add_sized([60.0, 22.0], egui::Label::new(RichText::new("Constant").strong().size(12.0)));
                ui.add_sized([ui.available_width() - 8.0, 22.0], egui::Label::new(RichText::new("Content preview").strong().size(12.0)));
            });
            ui.separator();

            for e in entries.iter_mut() {
                let row_response = ui.horizontal(|ui| {
                    let is_selected = *selected == Some(e.uid);
                if is_selected {
                    let rect = ui.max_rect();
                    ui.painter().rect_filled(rect, egui::CornerRadius::same(4), ui.style().visuals.selection.bg_fill);
                }
                    if ui.add_sized([60.0, 22.0], egui::Checkbox::new(&mut e.enabled, "")).changed() {}
                    ui.add_sized([44.0, 22.0], egui::Label::new(RichText::new(format!("{}", e.uid)).size(12.0)));
                    let mut name = e.name.clone();
                    let r = ui.add_sized([160.0, 22.0], TextEdit::singleline(&mut name).clip_text(true).font(egui::FontId::proportional(13.0)));
                    if r.changed() && name != e.name {
                        e.name = name;
                    }
                    let mut keys_str = e.keys.join(", ");
                    let r = ui.add_sized(
                        [240.0, 22.0],
                        TextEdit::singleline(&mut keys_str)
                            .clip_text(true)
                            .hint_text("comma, separated")
                            .font(egui::FontId::proportional(12.0)),
                    );
                    if r.changed() {
                        let new_keys: Vec<String> = keys_str.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if new_keys != e.keys {
                            e.keys = new_keys.clone();
                            e.key = new_keys;
                        }
                    }
                    let mut pri = e.priority as i32;
                    if ui.add_sized([70.0, 22.0], egui::DragValue::new(&mut pri).range(0..=1000).speed(1.0)).changed() {
                        e.priority = pri.max(0) as u32;
                        e.order = e.priority;
                        e.insertion_order = e.priority;
                        e.extensions.weight = e.priority;
                    }
                    let mut pos = e.position as i32;
                    if ui.add_sized([60.0, 22.0], egui::DragValue::new(&mut pos).range(0..=4)).changed() {
                        e.position = pos.max(0).min(255) as u8;
                    }
                    let _ = ui.add_sized([60.0, 22.0], egui::Checkbox::new(&mut e.selective, ""));
                    let _ = ui.add_sized([60.0, 22.0], egui::Checkbox::new(&mut e.constant, ""));
                    let mut preview: String = e.content.chars().take(120).collect();
                    if e.content.chars().count() > 120 { preview.push('…'); }
                    let r = ui.add_sized(
                        [ui.available_width() - 8.0, 22.0],
                        TextEdit::singleline(&mut preview)
                            .clip_text(true)
                            .hint_text("(double-click row to edit content)")
                            .font(egui::FontId::proportional(12.0)),
                    );
                    if r.double_clicked() {
                        content_editor = Some((e.uid, e.content.clone()));
                    }
                });
                let resp = row_response.response;
                if resp.clicked() { *selected = Some(e.uid); }
                if resp.double_clicked() {
                    content_editor = Some((e.uid, e.content.clone()));
                }
                if resp.secondary_clicked() { to_remove.push(e.uid); }
            }
        });

    for uid in to_remove {
        if let Some(idx) = entries.iter().position(|e| e.uid == uid) {
            entries.remove(idx);
            if *selected == Some(uid) { *selected = None; }
        }
    }

    if let Some((uid, mut content)) = content_editor {
        let mut open = true;
        let title = format!("Content for entry #{}", uid);
        egui::Window::new(title)
            .open(&mut open)
            .resizable(true)
            .default_size([640.0, 420.0])
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut content)
                            .font(egui::FontId::monospace(13.0))
                            .desired_width(f32::INFINITY)
                            .desired_rows(20),
                    );
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("💾  Save").clicked() {
                        for e in entries.iter_mut() {
                            if e.uid == uid { e.content = content.clone(); }
                        }
                    }
                    if ui.button("Cancel").clicked() { /* just close */ }
                });
            });
    }
}
