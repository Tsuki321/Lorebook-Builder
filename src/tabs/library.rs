use eframe::egui::{self, Color32, RichText, TextEdit, Ui};

use crate::model::{Store, WorldInfoEntry};
use crate::ui::{editor, table, widgets};

pub struct LibraryState {
    pub entries: Vec<WorldInfoEntry>,
    pub selected: Option<u64>,
    pub search: String,
    pub view: ViewMode,
    pub new_entry_name: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Table,
    SideEdit,
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            search: String::new(),
            view: ViewMode::Table,
            new_entry_name: String::new(),
            error: None,
        }
    }

    pub fn reload(&mut self, store: &Store) {
        match store.list_all() {
            Ok(raw) => match store.hydrate_all(raw) {
                Ok(entries) => { self.entries = entries; self.error = None; }
                Err(e) => { self.error = Some(format!("hydrate: {e:#}")); }
            }
            Err(e) => { self.error = Some(format!("load: {e:#}")); }
        }
    }
}

pub fn draw(ui: &mut Ui, state: &mut LibraryState, store: &Store) {
    widgets::section_header(
        ui,
        "Library",
        Some("Review, edit, and curate the entries you've crawled."),
    );

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("🔄  Reload from DB").clicked() { state.reload(store); }
        if ui.button("➕  Add row").clicked() {
            let uid = store.max_uid().unwrap_or(0) + 1;
            let name = if state.new_entry_name.is_empty() {
                format!("New entry #{uid}")
            } else {
                state.new_entry_name.clone()
            };
            let e = WorldInfoEntry::new(uid, name, Vec::new(), String::new(), String::new(), 50, 1);
            if let Err(e) = store.upsert_entry(&e, 0, 0, "", "") {
                state.error = Some(format!("add: {e:#}"));
            } else {
                state.entries.push(e);
            }
            state.new_entry_name.clear();
        }
        let mut new_name = state.new_entry_name.clone();
        if ui.add_sized([160.0, 24.0], TextEdit::singleline(&mut new_name).hint_text("New row name")).changed() {
            state.new_entry_name = new_name;
        }
        ui.separator();
        ui.label("Search:");
        let mut q = state.search.clone();
        if ui.add_sized([220.0, 24.0], TextEdit::singleline(&mut q).hint_text("name / key / content")).changed() {
            state.search = q;
        }
        ui.separator();
        ui.selectable_value(&mut state.view, ViewMode::Table, "📋  Table");
        ui.selectable_value(&mut state.view, ViewMode::SideEdit, "📝  Side editor");
    });

    if let Some(err) = &state.error {
        ui.colored_label(Color32::from_rgb(255, 100, 100), format!("⚠ {err}"));
    }

    ui.add_space(4.0);
    ui.label(format!("{} entries in library", state.entries.len()));
    ui.separator();

    let q = state.search.to_ascii_lowercase();
    let filtered: Vec<WorldInfoEntry> = if q.is_empty() {
        state.entries.clone()
    } else {
        state.entries.iter()
            .filter(|e| {
                e.name.to_ascii_lowercase().contains(&q)
                    || e.comment.to_ascii_lowercase().contains(&q)
                    || e.content.to_ascii_lowercase().contains(&q)
                    || e.keys.iter().any(|k| k.to_ascii_lowercase().contains(&q))
            })
            .cloned()
            .collect()
    };

    match state.view {
        ViewMode::Table => {
            let mut entries_mut = filtered;
            table::draw(ui, &mut entries_mut, &mut state.selected);
            for e in &entries_mut {
                if let Some(orig) = state.entries.iter_mut().find(|x| x.uid == e.uid) {
                    if orig != e {
                        *orig = e.clone();
                        let _ = store.upsert_entry(e, 0, 0, "", "");
                    }
                }
            }
        }
        ViewMode::SideEdit => {
            ui.horizontal(|ui| {
                ui.allocate_ui([280.0, ui.available_height()], |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for e in state.entries.iter() {
                            let label = format!("{}{}",
                                if e.enabled { "● " } else { "○ " },
                                e.name
                            );
                            if ui.selectable_label(state.selected == Some(e.uid), label).clicked() {
                                state.selected = Some(e.uid);
                            }
                        }
                    });
                });
                ui.separator();
                ui.allocate_ui(ui.available_size(), |ui| {
                    if let Some(uid) = state.selected {
                        if let Some(e) = state.entries.iter_mut().find(|e| e.uid == uid) {
                            if editor::draw(ui, e) {
                                let _ = store.upsert_entry(e, 0, 0, "", "");
                            }
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("🗑  Delete entry").clicked() {
                                    if let Err(err) = store.delete_entry(uid) {
                                        state.error = Some(format!("delete: {err:#}"));
                                    } else {
                                        state.entries.retain(|x| x.uid != uid);
                                        state.selected = None;
                                    }
                                }
                            });
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(RichText::new("Select an entry on the left to edit.").weak());
                        });
                    }
                });
            });
        }
    }
}
