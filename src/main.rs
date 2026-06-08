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
        .with_app_id("com.LorebookBuilder.app")
        .with_icon(app_icon());
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

/// Generate a 64x64 RGBA app icon at runtime: a Catppuccin-Mocha-blue
/// rounded square with a simple open-book glyph. No external image deps
/// required — pure math.
fn app_icon() -> egui::IconData {
    const SIZE: u32 = 64;
    const PAD: u32 = 4; // rounded-corner radius
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    let bg_top    = (0x89, 0xb4, 0xfa); // Catppuccin Mocha Blue
    let bg_bottom = (0x74, 0x9c, 0xe6);
    let page      = (0xcd, 0xd6, 0xf4); // Catppuccin Text
    let spine     = (0x1e, 0x1e, 0x2e); // Catppuccin Crust

    for y in 0..SIZE {
        for x in 0..SIZE {
            // Rounded-rect mask
            let in_left   = x >= PAD || y >= PAD && y < SIZE - PAD;
            let in_right  = x < SIZE - PAD || y >= PAD && y < SIZE - PAD;
            let in_corner_tl = x < PAD && y < PAD && (PAD - x).pow(2) + (PAD - y).pow(2) > PAD.pow(2);
            let in_corner_tr = x >= SIZE - PAD && y < PAD && (x - (SIZE - PAD - 1)).pow(2) + (PAD - y).pow(2) > PAD.pow(2);
            let in_corner_bl = x < PAD && y >= SIZE - PAD && (PAD - x).pow(2) + (y - (SIZE - PAD - 1)).pow(2) > PAD.pow(2);
            let in_corner_br = x >= SIZE - PAD && y >= SIZE - PAD && (x - (SIZE - PAD - 1)).pow(2) + (y - (SIZE - PAD - 1)).pow(2) > PAD.pow(2);
            if !(in_left && in_right) || in_corner_tl || in_corner_tr || in_corner_bl || in_corner_br {
                continue; // transparent
            }

            // Vertical gradient background
            let t = y as f32 / SIZE as f32;
            let r = (bg_top.0 as f32 * (1.0 - t) + bg_bottom.0 as f32 * t) as u8;
            let g = (bg_top.1 as f32 * (1.0 - t) + bg_bottom.1 as f32 * t) as u8;
            let b = (bg_top.2 as f32 * (1.0 - t) + bg_bottom.2 as f32 * t) as u8;

            // Open-book shape: two page rectangles left/right of a center spine
            let (r, g, b) = if (16..=30).contains(&x) && (24..=44).contains(&y) {
                // Left page
                (page.0, page.1, page.2)
            } else if (33..=47).contains(&x) && (24..=44).contains(&y) {
                // Right page
                (page.0, page.1, page.2)
            } else if (31..=33).contains(&x) && (22..=46).contains(&y) {
                // Spine
                (spine.0, spine.1, spine.2)
            } else {
                (r, g, b)
            };

            let i = ((y * SIZE + x) * 4) as usize;
            rgba[i]     = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
        }
    }

    egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}
