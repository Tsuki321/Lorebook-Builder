use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText, TextEdit, Ui};

use crate::export;
use crate::model::{Store, WorldInfo};
use crate::ui::widgets;

pub struct ExportState {
    pub output_path: PathBuf,
    pub name: String,
    pub description: String,
    pub scan_depth: u32,
    pub token_budget: u32,
    pub recursive_scanning: bool,
    pub default_priority: u32,
    pub default_probability: u8,
    pub default_depth: u32,
    pub default_selective: bool,
    pub default_constant: bool,
    pub message: Option<(Color32, String)>,
}

impl ExportState {
    pub fn new() -> Self {
        let default = directories::UserDirs::new()
            .map(|u| u.document_dir().unwrap_or_else(|| u.home_dir()).join("lorebook.json"))
            .unwrap_or_else(|| PathBuf::from("lorebook.json"));
        Self {
            output_path: default,
            name: "Lorebook".into(),
            description: String::new(),
            scan_depth: 50,
            token_budget: 2000,
            recursive_scanning: false,
            default_priority: 100,
            default_probability: 100,
            default_depth: 4,
            default_selective: true,
            default_constant: false,
            message: None,
        }
    }
}

pub fn draw(ui: &mut Ui, state: &mut ExportState, store: &Store) {
    widgets::section_header(
        ui,
        "Export",
        Some("Write a SillyTavern-compatible world info JSON file."),
    );

    ui.horizontal(|ui| {
        ui.label(RichText::new("Output file:").strong());
        ui.add_sized(
            [ui.available_width() - 90.0, 24.0],
            TextEdit::singleline(&mut state.output_path.to_string_lossy().to_string())
                .font(egui::FontId::proportional(13.0)),
        );
        if ui.button("📁").clicked() {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name(state.output_path.file_name().and_then(|s| s.to_str()).unwrap_or("lorebook.json"))
                .save_file() {
                state.output_path = p;
            }
        }
    });

    ui.add_space(6.0);
    ui.collapsing(RichText::new("Top-level").strong(), |ui| {
        ui.horizontal(|ui| { ui.label("Name"); ui.add_sized([300.0, 22.0], TextEdit::singleline(&mut state.name)); });
        ui.horizontal(|ui| { ui.label("Description"); ui.add_sized([300.0, 22.0], TextEdit::singleline(&mut state.description)); });
        ui.horizontal(|ui| { ui.label("Scan depth");
            ui.add(egui::DragValue::new(&mut state.scan_depth).range(0..=1000)); });
        ui.horizontal(|ui| { ui.label("Token budget");
            ui.add(egui::DragValue::new(&mut state.token_budget).range(0..=10_000)); });
        ui.checkbox(&mut state.recursive_scanning, "Recursive scanning");
    });

    ui.collapsing(RichText::new("Default per-entry").strong(), |ui| {
        ui.horizontal(|ui| { ui.label("Priority");
            ui.add(egui::DragValue::new(&mut state.default_priority).range(0..=1000)); });
        ui.horizontal(|ui| { ui.label("Probability");
            ui.add(egui::DragValue::new(&mut state.default_probability).range(0..=100)); });
        ui.horizontal(|ui| { ui.label("Depth");
            ui.add(egui::DragValue::new(&mut state.default_depth).range(0..=255)); });
        ui.checkbox(&mut state.default_selective, "Selective");
        ui.checkbox(&mut state.default_constant, "Constant");
    });

    ui.add_space(8.0);
    if ui.add_sized([220.0, 32.0], egui::Button::new(RichText::new("💾  Export SillyTavern JSON").strong())).clicked() {
        do_export(state, store);
    }

    if let Some((color, msg)) = &state.message {
        ui.add_space(6.0);
        ui.colored_label(*color, msg);
    }
}

fn do_export(state: &mut ExportState, store: &Store) {
    match build_world_info(state, store) {
        Ok(wb) => {
            let s = export::to_string_pretty(&wb);
            if let Err(e) = std::fs::write(&state.output_path, s) {
                state.message = Some((Color32::from_rgb(255, 100, 100), format!("❌ write failed: {e}")));
                return;
            }
            let n = wb.entries.len();
            state.message = Some((Color32::from_rgb(100, 200, 100),
                format!("✅ Exported {n} entries to {}", state.output_path.display())));
        }
        Err(e) => {
            state.message = Some((Color32::from_rgb(255, 100, 100), format!("❌ build failed: {e:#}")));
        }
    }
}

fn build_world_info(state: &ExportState, store: &Store) -> anyhow::Result<WorldInfo> {
    let mut wb = WorldInfo::new(state.name.clone());
    wb.description = state.description.clone();
    wb.scan_depth = state.scan_depth;
    wb.token_budget = state.token_budget;
    wb.recursive_scanning = state.recursive_scanning;

    let raw = store.list_all()?;
    let entries = store.hydrate_all(raw)?;
    for mut e in entries {
        // Apply defaults where the user left them at their default
        if e.priority == 0 { e.priority = state.default_priority; }
        if e.depth == 0 { e.depth = state.default_depth; }
        if e.probability == 0 { e.probability = state.default_probability; }
        // Always re-stamp order/insertion_order/extensions.weight to match priority
        e.order = e.priority;
        e.insertion_order = e.priority;
        e.extensions.weight = e.priority;
        e.extensions.depth = e.depth;
        e.extensions.probability = e.probability;
        wb.entries.insert(e.uid.to_string(), e);
    }
    Ok(wb)
}
