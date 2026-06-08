//! Hand-rolled icon renderer for the seven seed categories.
//!
//! Each icon is a stylised approximation of its SVG counterpart in
//! `assets/icons/`. We use raw `egui::Painter` primitives (circles,
//! rectangles, lines, paths) so the binary stays dependency-free — no
//! `resvg`, `tiny-skia`, or `usvg` runtime needed.

use eframe::egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

/// Allocate space for and draw a 16×16 icon for the given category.
/// Falls back to a simple dot for unknown names.
pub fn draw(ui: &mut Ui, name: &str) {
    let size = 16.0_f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(size, size), Sense::hover());
    let color = ui.style().visuals.text_color();
    let painter = ui.painter();
    let stroke = Stroke::new(1.4, color);
    match name {
        "Characters"    => character(painter, rect, color, stroke),
        "Locations"     => location(painter, rect, color, stroke),
        "Pathways"      => pathway(painter, rect, color, stroke),
        "Items"         => item(painter, rect, color, stroke),
        "Organizations" => organization(painter, rect, color, stroke),
        "Events"        => event(painter, rect, color, stroke),
        "Terminology"   => terminology(painter, rect, color, stroke),
        _ => {
            painter.circle_filled(rect.center(), 2.0, color);
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

fn path_closed(painter: &egui::Painter, points: Vec<Pos2>, fill: Color32, stroke: Stroke) {
    painter.add(Shape::Path(egui::epaint::PathShape {
        points,
        closed: true,
        fill,
        stroke: stroke.into(),
    }));
}

// ---- category icons --------------------------------------------------------

/// Person silhouette: head + body.
fn character(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    // Head
    p.circle_filled(Pos2::new(r.center().x, r.top() + 4.0), 2.6, c);
    // Shoulders: open curve approximated with 4 line segments
    let shoulder_y = r.top() + 8.0;
    line(p, Pos2::new(r.left() + 3.0, r.bottom() - 1.0),
            Pos2::new(r.left() + 3.0, shoulder_y + 1.0), s);
    line(p, Pos2::new(r.left() + 3.0, shoulder_y + 1.0),
            Pos2::new(r.center().x, shoulder_y - 0.5), s);
    line(p, Pos2::new(r.center().x, shoulder_y - 0.5),
            Pos2::new(r.right() - 3.0, shoulder_y + 1.0), s);
    line(p, Pos2::new(r.right() - 3.0, shoulder_y + 1.0),
            Pos2::new(r.right() - 3.0, r.bottom() - 1.0), s);
}

/// Map pin: teardrop + inner dot.
fn location(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    let cx = r.center().x;
    let circle_center = Pos2::new(cx, r.top() + 4.5);
    p.circle_stroke(circle_center, 4.0, s);
    // Tail of the pin
    let pts = vec![
        Pos2::new(cx - 3.5, r.top() + 7.0),
        Pos2::new(cx, r.bottom() - 1.0),
        Pos2::new(cx + 3.5, r.top() + 7.0),
    ];
    path_open(p, pts, s);
    p.circle_filled(circle_center, 1.2, c);
}

/// Winding path: a sine-like curve from left to right.
fn pathway(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    let pts: Vec<Pos2> = (0..=16)
        .map(|i| {
            let t = i as f32 / 16.0;
            let x = r.left() + t * r.width();
            let y = r.center().y + (t * std::f32::consts::TAU).sin() * 4.0;
            Pos2::new(x, y)
        })
        .collect();
    path_open(p, pts.clone(), s);
    // Start and end dots
    if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
        p.circle_filled(*first, 1.4, c);
        p.circle_filled(*last, 1.4, c);
    }
}

/// Sword: a diagonal blade + crossguard.
fn item(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    // Blade
    line(p,
        Pos2::new(r.left() + 3.0, r.top() + 3.0),
        Pos2::new(r.right() - 3.0, r.bottom() - 3.0),
        s);
    // Crossguard
    line(p,
        Pos2::new(r.left() + 1.5, r.bottom() - 3.0),
        Pos2::new(r.left() + 4.5, r.bottom() - 6.0),
        s);
    // Pommel
    p.circle_filled(Pos2::new(r.left() + 1.5, r.bottom() - 3.0), 1.0, c);
}

/// Building with columns: house silhouette + door.
fn organization(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    // Roof
    let pts = vec![
        Pos2::new(r.left() + 1.5, r.top() + 6.0),
        Pos2::new(r.center().x, r.top() + 1.5),
        Pos2::new(r.right() - 1.5, r.top() + 6.0),
    ];
    path_open(p, pts, s);
    // Walls
    line(p, Pos2::new(r.left() + 1.5, r.top() + 6.0), Pos2::new(r.left() + 1.5, r.bottom() - 1.0), s);
    line(p, Pos2::new(r.right() - 1.5, r.top() + 6.0), Pos2::new(r.right() - 1.5, r.bottom() - 1.0), s);
    // Floor
    line(p, Pos2::new(r.left() + 0.5, r.bottom() - 1.0), Pos2::new(r.right() - 0.5, r.bottom() - 1.0), s);
    // Door
    let door = Rect::from_min_max(
        Pos2::new(r.center().x - 2.0, r.top() + 9.0),
        Pos2::new(r.center().x + 2.0, r.bottom() - 1.0),
    );
    p.rect_filled(door, 0.0, tint(c));
}

/// Star burst: one bigger and one smaller 8-pointed star.
fn event(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    let cx = r.center().x;
    let cy = r.top() + 5.5;
    let arm = 4.0;
    let big = vec![
        Pos2::new(cx, cy - arm),
        Pos2::new(cx + 1.2, cy - 1.2),
        Pos2::new(cx + arm, cy),
        Pos2::new(cx + 1.2, cy + 1.2),
        Pos2::new(cx, cy + arm),
        Pos2::new(cx - 1.2, cy + 1.2),
        Pos2::new(cx - arm, cy),
        Pos2::new(cx - 1.2, cy - 1.2),
        Pos2::new(cx, cy - arm),
    ];
    path_closed(p, big, c, s);

    let sx = r.right() - 3.0;
    let sy = r.bottom() - 3.0;
    let arm2 = 2.0;
    let small = vec![
        Pos2::new(sx, sy - arm2),
        Pos2::new(sx + 0.5, sy - 0.5),
        Pos2::new(sx + arm2, sy),
        Pos2::new(sx + 0.5, sy + 0.5),
        Pos2::new(sx, sy + arm2),
        Pos2::new(sx - 0.5, sy + 0.5),
        Pos2::new(sx - arm2, sy),
        Pos2::new(sx - 0.5, sy - 0.5),
        Pos2::new(sx, sy - arm2),
    ];
    path_closed(p, small, c, s);
}

/// Open book with two pages.
fn terminology(p: &egui::Painter, r: Rect, c: Color32, s: Stroke) {
    // Spine
    line(p,
        Pos2::new(r.center().x, r.top() + 1.5),
        Pos2::new(r.center().x, r.bottom() - 1.5),
        s);
    // Left page
    let left = Rect::from_min_max(
        Pos2::new(r.left() + 1.0, r.top() + 1.5),
        Pos2::new(r.center().x - 0.5, r.bottom() - 1.5),
    );
    p.rect_stroke(left, 0.0, s, StrokeKind::Inside);
    // Right page
    let right = Rect::from_min_max(
        Pos2::new(r.center().x + 0.5, r.top() + 1.5),
        Pos2::new(r.right() - 1.0, r.bottom() - 1.5),
    );
    p.rect_stroke(right, 0.0, s, StrokeKind::Inside);
    // Two lines of "text" on each page
    let dim = tint(c);
    for y_off in [4.0_f32, 7.0] {
        line(p,
            Pos2::new(left.left() + 1.5, left.top() + y_off),
            Pos2::new(left.right() - 1.5, left.top() + y_off),
            Stroke::new(0.8, dim));
        line(p,
            Pos2::new(right.left() + 1.5, right.top() + y_off),
            Pos2::new(right.right() - 1.5, right.top() + y_off),
            Stroke::new(0.8, dim));
    }
}

fn tint(c: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90)
}
