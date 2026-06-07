use eframe::egui::{self, Color32, RichText, TextEdit, Ui};

use crate::model::{Store, WorldInfoEntry};
use crate::ui::toast::ToastQueue;
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
        match store.list_all_with_keys() {
            Ok(entries) => { self.entries = entries; self.error = None; }
            Err(e) => { self.error = Some(format!("load: {e:#}")); }
        }
    }
}

pub fn draw(ui: &mut Ui, state: &mut LibraryState, store: &Store, toasts: &mut ToastQueue) {
    widgets::section_header(
        ui,
        "Library",
        Some("Review, edit, and curate the entries you've crawled."),
    );

    // Toolbar
    ui.horizontal_wrapped(|ui| {
        if ui.button("🔄  Reload").clicked() { state.reload(store); }
        if ui.button("➕  Add row").clicked() {
            let uid = store.max_uid().unwrap_or(0) + 1;
            let name = if state.new_entry_name.is_empty() {
                format!("New entry #{uid}")
            } else {
                state.new_entry_name.clone()
            };
            let e = WorldInfoEntry::new(uid, name, Vec::new(), String::new(), String::new(), 50, 1);
            match store.upsert_entry(&e, 0, 0, "", "") {
                Ok(()) => {
                    state.entries.push(e);
                    toasts.success(format!("Added entry #{uid}"));
                }
                Err(err) => {
                    state.error = Some(format!("add: {err:#}"));
                    toasts.error(format!("Add failed: {err:#}"));
                }
            }
            state.new_entry_name.clear();
        }
        let mut new_name = state.new_entry_name.clone();
        if ui.add_sized([140.0, 24.0], TextEdit::singleline(&mut new_name).hint_text("New row name")).changed() {
            state.new_entry_name = new_name;
        }
        ui.separator();
        ui.label("Search:");
        let mut q = state.search.clone();
        if ui.add_sized([200.0, 24.0], TextEdit::singleline(&mut q).hint_text("name / key / content")).changed() {
            state.search = q;
        }
        ui.separator();
        if ui.add_enabled(state.selected.is_some(), egui::Button::new("⎘  Duplicate"))
            .on_hover_text("Make a copy of the selected entry")
            .clicked() {
            duplicate_selected(state, store, toasts);
        }
        if ui.add_enabled(state.selected.is_some(), egui::Button::new("🗑  Delete"))
            .on_hover_text("Delete the selected entry from the library")
            .clicked() {
            delete_selected(state, store, toasts);
        }
        if ui.add_enabled(!state.entries.is_empty(), egui::Button::new("🧹  Clear all"))
            .on_hover_text("Delete every entry from the library")
            .clicked() {
            clear_all(state, store, toasts);
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
                ui.allocate_ui(egui::vec2(280.0, ui.available_height()), |ui| {
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
                                if ui.button("⎘  Duplicate").clicked() {
                                    duplicate_selected(state, store, toasts);
                                }
                                if ui.button("🗑  Delete entry").clicked() {
                                    delete_selected(state, store, toasts);
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

fn duplicate_selected(state: &mut LibraryState, store: &Store, toasts: &mut ToastQueue) {
    let Some(uid) = state.selected else { return; };
    let Some(src) = state.entries.iter().find(|e| e.uid == uid).cloned() else { return; };
    let new_uid = store.max_uid().unwrap_or(0) + 1;
    let mut copy = src.clone();
    copy.uid = new_uid;
    copy.name = format!("{} (copy)", src.name);
    copy.order = new_uid as u32;
    copy.insertion_order = new_uid as u32;
    match store.upsert_entry(&copy, 0, 0, "", "") {
        Ok(()) => {
            state.entries.push(copy);
            state.selected = Some(new_uid);
            toasts.success(format!("Duplicated as #{new_uid}"));
        }
        Err(err) => {
            state.error = Some(format!("duplicate: {err:#}"));
            toasts.error(format!("Duplicate failed: {err:#}"));
        }
    }
}

fn delete_selected(state: &mut LibraryState, store: &Store, toasts: &mut ToastQueue) {
    let Some(uid) = state.selected else { return; };
    let name = state.entries.iter().find(|e| e.uid == uid).map(|e| e.name.clone()).unwrap_or_default();
    match store.delete_entry(uid) {
        Ok(()) => {
            state.entries.retain(|x| x.uid != uid);
            state.selected = None;
            toasts.success(format!("Deleted '{}'", name));
        }
        Err(err) => {
            state.error = Some(format!("delete: {err:#}"));
            toasts.error(format!("Delete failed: {err:#}"));
        }
    }
}

fn clear_all(state: &mut LibraryState, store: &Store, toasts: &mut ToastQueue) {
    match store.clear_all() {
        Ok(n) => {
            state.entries.clear();
            state.selected = None;
            toasts.warn(format!("Cleared {n} entries from library"));
        }
        Err(err) => {
            state.error = Some(format!("clear: {err:#}"));
            toasts.error(format!("Clear failed: {err:#}"));
        }
    }
}
