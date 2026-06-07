use std::path::PathBuf;

use eframe::egui::{self, Context, RichText, TopBottomPanel};
use parking_lot::Mutex;

use crate::crawler::ProgressEvent;
use crate::model::Store;
use crate::tabs;
use crate::theme::{self, ThemeChoice};
use crate::ui::toast::{self, ToastQueue};
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Crawl,
    Library,
    Export,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Crawl => "Crawl",
            Tab::Library => "Library",
            Tab::Export => "Export",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Tab::Crawl => "🜲",
            Tab::Library => "🕮",
            Tab::Export => "⇪",
        }
    }
}

pub struct App {
    pub active_tab: Tab,
    pub theme: ThemeChoice,
    pub store: Mutex<Store>,
    pub crawl_state: tabs::crawl::CrawlState,
    pub library_state: tabs::library::LibraryState,
    pub export_state: tabs::export::ExportState,
    pub toasts: ToastQueue,
    pub entries_count_cache: u64,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, store: Store) -> Self {
        theme::apply(&cc.egui_ctx, ThemeChoice::Mocha);
        let lib = tabs::library::LibraryState::new();
        let mut s = Self {
            active_tab: Tab::Crawl,
            theme: ThemeChoice::Mocha,
            store: Mutex::new(store),
            crawl_state: tabs::crawl::CrawlState::new(),
            library_state: lib,
            export_state: tabs::export::ExportState::new(),
            toasts: ToastQueue::new(),
            entries_count_cache: 0,
        };
        s.refresh_entries_count();
        s
    }

    pub fn refresh_entries_count(&mut self) {
        if let Ok(s) = self.store.lock().count() {
            self.entries_count_cache = s;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.show_top_bar(ctx);
        self.show_status_bar(ctx);
        self.show_sidebar(ctx);
        self.show_central(ctx);
        self.drain_crawl_events();
        self.handle_shortcuts(ctx);

        self.toasts.tick();
        toast::render(ctx, &self.toasts);

        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}

impl App {
    fn show_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("topbar")
            .frame(egui::Frame::new()
                .fill(ctx.style().visuals.window_fill)
                .inner_margin(egui::Margin::symmetric(14, 8)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📖 Lorebook Builder").size(18.0).strong());
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(format!("{} entries", self.entries_count_cache));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("theme-picker")
                            .selected_text(self.theme.display())
                            .show_ui(ui, |ui| {
                                for t in ThemeChoice::all() {
                                    if ui.selectable_label(self.theme == t, t.display()).clicked() {
                                        self.theme = t;
                                        theme::apply(ctx, t);
                                    }
                                }
                                None::<()>
                            });
                    });
                });
            });
    }

    fn show_status_bar(&self, ctx: &Context) {
        TopBottomPanel::bottom("statusbar")
            .frame(egui::Frame::new()
                .fill(ctx.style().visuals.window_fill)
                .inner_margin(egui::Margin::symmetric(14, 6)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dot = if self.crawl_state.running { "🟡" } else { "🟢" };
                    let status = if self.crawl_state.running { "Crawler: running" } else { "Crawler: idle" };
                    ui.label(RichText::new(format!("{dot} {status}")).size(12.0));
                    ui.add_space(12.0);
                    if self.crawl_state.running {
                        ui.label(RichText::new(format!("OK {} / Err {} / Cached {}",
                            self.crawl_state.ok, self.crawl_state.err, self.crawl_state.cached)).size(12.0).weak());
                    } else if !self.crawl_state.last_message.is_empty() {
                        ui.label(RichText::new(&self.crawl_state.last_message).size(12.0).weak());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to("v0.1.0 · MIT", "https://github.com/Tsuki321/Lorebook-Builder");
                    });
                });
            });
    }

    fn show_sidebar(&mut self, ctx: &Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(180.0)
            .frame(egui::Frame::new()
                .fill(ctx.style().visuals.window_fill)
                .inner_margin(egui::Margin::same(8)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                for t in [Tab::Crawl, Tab::Library, Tab::Export] {
                    let badge = match t {
                        Tab::Library => {
                            if self.entries_count_cache > 0 { Some(self.entries_count_cache.to_string()) } else { None }
                        }
                        _ => None,
                    };
                    if crate::ui::widgets::sidebar_button(ui, t.label(), t.icon(), self.active_tab == t, badge) {
                        self.active_tab = t;
                    }
                }
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(RichText::new("Shortcuts").size(12.0).strong().weak());
                ui.label(RichText::new("Ctrl+1  Crawl").size(11.0).weak());
                ui.label(RichText::new("Ctrl+2  Library").size(11.0).weak());
                ui.label(RichText::new("Ctrl+3  Export").size(11.0).weak());
            });
    }

    fn show_central(&mut self, ctx: &Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(ctx.style().visuals.faint_bg_color)
                .inner_margin(egui::Margin::same(14)))
            .show(ctx, |ui| {
                let store = self.store.lock();
                match self.active_tab {
                    Tab::Crawl => {
                        let _ = tabs::crawl::draw(ui, &mut self.crawl_state, &store, &mut self.toasts);
                    }
                    Tab::Library => {
                        tabs::library::draw(ui, &mut self.library_state, &store, &mut self.toasts);
                    }
                    Tab::Export => {
                        tabs::export::draw(ui, &mut self.export_state, &store, &mut self.toasts);
                    }
                }
                drop(store);
                if matches!(self.active_tab, Tab::Library) {
                    self.refresh_entries_count();
                }
            });
    }

    fn drain_crawl_events(&mut self) {
        while let Some(ev) = self.crawl_state.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.handle_progress_event(ev);
        }
    }

    fn handle_progress_event(&mut self, ev: ProgressEvent) {
        match ev {
            ProgressEvent::Done => {
                self.refresh_entries_count();
                info!(
                    built = self.crawl_state.entries_built,
                    errors = self.crawl_state.err,
                    ok = self.crawl_state.ok,
                    cached = self.crawl_state.cached,
                    "crawl complete"
                );
                self.toasts.success(format!(
                    "Crawl complete — {} entries, {} errors",
                    self.crawl_state.entries_built, self.crawl_state.err
                ));
            }
            ProgressEvent::Cancelled => {
                self.refresh_entries_count();
                info!(
                    built = self.crawl_state.entries_built,
                    "crawl cancelled"
                );
                self.crawl_state.last_message = "Cancelled.".into();
                self.toasts.warn("Crawl cancelled.");
            }
            ProgressEvent::PageFetched { title, cached, .. } => {
                if cached { self.crawl_state.cached += 1; } else { self.crawl_state.ok += 1; }
                debug!(title = %title, cached, "page fetched");
                self.crawl_state.log.push((now_hms_c(), format!("Fetched {title}")));
            }
            ProgressEvent::CategoryEntered { title, total_in } => {
                self.crawl_state.total_in = self.crawl_state.total_in.saturating_add(total_in as u64);
                debug!(title = %title, total_in, "category entered");
                self.crawl_state.log.push((now_hms_c(), format!("→ Category {title} ({total_in} pages)")));
            }
            ProgressEvent::EntryBuilt { name, .. } => {
                self.crawl_state.entries_built += 1;
                debug!(name = %name, "entry built");
                self.crawl_state.log.push((now_hms_c(), format!("Stored entry: {name}")));
                self.refresh_entries_count();
            }
            ProgressEvent::PageFailed { title, error } => {
                self.crawl_state.err += 1;
                self.crawl_state.log.push((now_hms_c(), format!("FAIL {title}: {error}")));
                self.crawl_state.last_message = format!("Error: {title}");
                self.toasts.error(format!("{title}: {error}"));
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.command) { self.active_tab = Tab::Crawl; }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2) && i.modifiers.command) { self.active_tab = Tab::Library; }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3) && i.modifiers.command) { self.active_tab = Tab::Export; }
    }
}

pub fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "LorebookBuilder", "LorebookBuilder")
        .map(|p| p.data_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

fn now_hms_c() -> String {
    let t = chrono::Local::now();
    t.format("%H:%M:%S").to_string()
}
