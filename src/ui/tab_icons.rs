//! Hand-rolled icon renderer for the three sidebar tabs.
//!
//! Each icon is a stylised approximation of its SVG counterpart in
//! `assets/icons/tabs/`. We use raw `egui::Painter` primitives
//! (circles, rectangles, lines, paths) so the binary stays
//! dependency-free — no `resvg`, `tiny-skia`, or `usvg` runtime needed.

use eframe::egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

/// Allocate space for and draw an 18×18 icon for the given tab key.
/// Falls back to a simple dot for unknown keys.
pub fn draw(ui: &mut Ui, name: &str) {
    let size = 18.0_f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(size, size), Sense::hover());
    let color = ui.style().visuals.text_color();
    draw_in_rect(ui.painter(), name, rect, color);
}

/// Draw the icon for `name` into the given `rect`, using the given
/// explicit color (so callers like the sidebar can pass a custom
/// text color that doesn't necessarily match `visuals.text_color()`).
pub fn draw_in_rect(painter: &egui::Painter, name: &str, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    match name {
        "crawl"   => crawl(painter, rect, color, stroke),
        "library" => library(painter, rect, color, stroke),
        "export"  => export(painter, rect, color, stroke),
        _ => {
            painter.circle_filled(rect.center(), 1.8, color);
        }
    }
}

// ---- helpers ---------------------------------------------------------------

fn line(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke) {
    painter.line_segment([a, b], stroke);
}

fn path_open(painter: &egui::Painter, points: Vec<Pos2>, stroke: Stroke) {
    painter.add(Shape::Path(egui::epaint::PathShape {
        points,
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: stroke.into(),
    }));
}

// ---- tab icons -------------------------------------------------------------

/// Crawl: down arrow into a tray (download).
fn crawl(p: &egui::Painter, r: Rect, _c: Color32, s: Stroke) {
    let cx = r.center().x;
    // Vertical shaft
    line(p, Pos2::new(cx, r.top() + 1.5), Pos2::new(cx, r.bottom() - 5.5), s);
    // Arrow head (V)
    line(p, Pos2::new(cx, r.bottom() - 5.5),
            Pos2::new(cx - 4.0, r.bottom() - 9.5), s);
    line(p, Pos2::new(cx, r.bottom() - 5.5),
            Pos2::new(cx + 4.0, r.bottom() - 9.5), s);
    // Tray
    line(p, Pos2::new(r.left() + 1.0, r.bottom() - 1.5),
            Pos2::new(r.right() - 1.0, r.bottom() - 1.5), s);
}

/// Library: three stacked books with title lines.
fn library(p: &egui::Painter, r: Rect, _c: Color32, s: Stroke) {
    let book_h = 3.6;
    let gap = 0.6;
    let total_h = book_h * 3.0 + gap * 2.0;
    let start_y = r.top() + (r.height() - total_h) * 0.5;
    let widths = [4.0_f32, 2.5, 3.5]; // varying title-line lengths

    for i in 0..3 {
        let y = start_y + i as f32 * (book_h + gap);
        let book = Rect::from_min_max(
            Pos2::new(r.left() + 1.5, y),
            Pos2::new(r.right() - 1.5, y + book_h),
        );
        p.rect_stroke(book, 1.0, s, StrokeKind::Inside);
        // Title line, varying length per book
        let line_y = y + book_h * 0.55;
        line(p,
            Pos2::new(book.left() + 1.5, line_y),
            Pos2::new(book.left() + 1.5 + widths[i], line_y),
            s);
    }
}

/// Export: page with folded corner + up arrow.
fn export(p: &egui::Painter, r: Rect, _c: Color32, s: Stroke) {
    // Document outline (with room for the folded corner)
    let doc = Rect::from_min_max(
        Pos2::new(r.left() + 3.0, r.top() + 1.0),
        Pos2::new(r.right() - 1.5, r.bottom() - 1.0),
    );
    let fold = 4.0_f32;
    // Path: down the left side, across the bottom, up the right, then
    // diagonal to the corner of the fold, then the fold crease.
    let pts = vec![
        Pos2::new(doc.left(), doc.top()),
        Pos2::new(doc.left(), doc.bottom()),
        Pos2::new(doc.right(), doc.bottom()),
        Pos2::new(doc.right(), doc.top() + fold),
        Pos2::new(doc.right() - fold, doc.top()),
        Pos2::new(doc.left(), doc.top()),
    ];
    path_open(p, pts, s);
    // Folded-corner diagonal (the crease)
    line(p,
        Pos2::new(doc.right() - fold, doc.top()),
        Pos2::new(doc.right() - fold, doc.top() + fold),
        s);
    line(p,
        Pos2::new(doc.right() - fold, doc.top() + fold),
        Pos2::new(doc.right(), doc.top() + fold),
        s);

    // Up arrow inside the document
    let cx = doc.center().x;
    let arrow_top    = doc.top() + 5.5;
    let arrow_bottom = doc.bottom() - 4.0;
    line(p, Pos2::new(cx, arrow_top), Pos2::new(cx, arrow_bottom), s);
    line(p, Pos2::new(cx - 2.5, arrow_top + 2.5), Pos2::new(cx, arrow_top), s);
    line(p, Pos2::new(cx + 2.5, arrow_top + 2.5), Pos2::new(cx, arrow_top), s);
}
