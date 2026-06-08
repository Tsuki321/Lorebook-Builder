# Tab icons

Source-of-truth SVG files for the three sidebar tabs in the app.
The Rust UI renders stylised equivalents in `src/ui/tab_icons.rs`
using `egui::Painter` primitives (no SVG runtime, no new deps).

| File        | Tab     | Meaning |
|-------------|---------|---------|
| crawl.svg   | Crawl   | Down-arrow into tray (download pages) |
| library.svg | Library | Three stacked books (your entries) |
| export.svg  | Export  | Page with up-arrow (write lorebook JSON) |
