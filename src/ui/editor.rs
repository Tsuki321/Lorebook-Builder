use eframe::egui::{self, RichText, TextEdit, Ui};

use crate::model::WorldInfoEntry;

/// Side editor panel for a single selected entry.
pub fn draw(ui: &mut Ui, entry: &mut WorldInfoEntry) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Name").strong());
        let r = ui.add(
            TextEdit::singleline(&mut entry.name)
                .desired_width(f32::INFINITY)
                .hint_text("Entry name (page title)"),
        );
        if r.changed() { changed = true; }
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new("Comment").strong());
        let r = ui.add(
            TextEdit::singleline(&mut entry.comment)
                .desired_width(f32::INFINITY)
                .hint_text("Notes about this entry"),
        );
        if r.changed() { changed = true; }
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Keys (primary)").strong());
    let mut keys_str = entry.keys.join(", ");
    let r = ui.add(
        TextEdit::singleline(&mut keys_str)
            .desired_width(f32::INFINITY)
            .hint_text("Comma-separated trigger words"),
    );
    if r.changed() {
        let new_keys: Vec<String> = keys_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if new_keys != entry.keys {
            entry.keys = new_keys.clone();
            entry.key = new_keys;
            changed = true;
        }
    }
    ui.label(RichText::new("Secondary keys").strong());
    let mut sec_str = entry.secondary_keys.join(", ");
    let r = ui.add(
        TextEdit::singleline(&mut sec_str)
            .desired_width(f32::INFINITY)
            .hint_text("Optional secondary triggers"),
    );
    if r.changed() {
        let new_keys: Vec<String> = sec_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if new_keys != entry.secondary_keys {
            entry.secondary_keys = new_keys.clone();
            entry.keysecondary = new_keys;
            changed = true;
        }
    }

    ui.add_space(6.0);
    ui.collapsing(RichText::new("Settings").strong(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Priority");
            let mut pri = entry.priority as i32;
            if ui.add(egui::DragValue::new(&mut pri).range(0..=1000).speed(1.0)).changed() {
                entry.priority = pri.max(0) as u32;
                entry.order = entry.priority;
                entry.insertion_order = entry.priority;
                entry.extensions.weight = entry.priority;
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Position");
            let mut pos = entry.position as i32;
            if ui.add(egui::DragValue::new(&mut pos).range(0..=4)).changed() {
                entry.position = pos.max(0).min(255) as u8;
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Probability");
            let mut p = entry.probability as i32;
            if ui.add(egui::DragValue::new(&mut p).range(0..=100)).changed() {
                entry.probability = p.max(0).min(255) as u8;
                entry.extensions.probability = entry.probability;
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Depth");
            let mut d = entry.depth as i32;
            if ui.add(egui::DragValue::new(&mut d).range(0..=255)).changed() {
                entry.depth = d.max(0) as u32;
                entry.extensions.depth = entry.depth;
                changed = true;
            }
        });
        if ui.checkbox(&mut entry.enabled, "Enabled").changed() { changed = true; }
        if ui.checkbox(&mut entry.constant, "Constant (fire on every scan)").changed() { changed = true; }
        if ui.checkbox(&mut entry.selective, "Selective (require key match)").changed() { changed = true; }
        if ui.checkbox(&mut entry.case_sensitive, "Case sensitive").changed() { changed = true; }
        if ui.checkbox(&mut entry.use_probability, "Use probability").changed() { changed = true; }
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Content").strong());
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            let r = ui.add(
                TextEdit::multiline(&mut entry.content)
                    .font(egui::FontId::monospace(13.0))
                    .desired_width(f32::INFINITY)
                    .desired_rows(20),
            );
            if r.changed() { changed = true; }
        });

    changed
}
