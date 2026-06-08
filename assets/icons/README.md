# Category icons

These are the source-of-truth SVG files for the seed-category icons
shown in the Crawl tab. The Rust UI renders stylised equivalents
in `src/ui/category_icons.rs` using `egui::Painter` primitives (no
SVG runtime needed, no new dependencies).

To re-render them in egui, the painter code is a hand-rolled
approximation of the SVG paths above. Keeping the SVGs here means
the design intent is preserved as a portable asset you can also
use in a README, GitHub social card, etc.

| File              | Category      |
|-------------------|---------------|
| characters.svg    | Characters    |
| locations.svg     | Locations     |
| pathways.svg      | Pathways      |
| items.svg         | Items         |
| organizations.svg | Organizations |
| events.svg        | Events        |
| terminology.svg   | Terminology   |
