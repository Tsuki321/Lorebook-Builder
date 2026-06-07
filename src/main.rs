#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod config;
mod crawler;
mod export;
mod model;
mod tabs;
mod theme;
mod ui;

use app::App;
use eframe::egui;
use model::Store;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let data_dir: PathBuf = app::data_dir();
    let _ = std::fs::create_dir_all(&data_dir);
    let db_path = data_dir.join("lorebook.sqlite");
    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("FATAL: cannot open store at {}: {e:#}", db_path.display());
            std::process::exit(2);
        }
    };

    tracing::info!(data_dir = %data_dir.display(), db_path = %db_path.display(), "Lorebook Builder starting");

    let viewport = egui::ViewportBuilder::default()
        .with_title("Lorebook Builder")
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([960.0, 640.0])
        .with_app_id("com.LorebookBuilder.app");
    let options = eframe::NativeOptions {
        viewport,
        vsync: true,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Lorebook Builder",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, store)))),
    )
}
