use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, Context, FontId, Sense, Stroke, StrokeKind, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    pub fn color(self) -> Color32 {
        match self {
            ToastKind::Info => Color32::from_rgb(120, 170, 240),
            ToastKind::Success => Color32::from_rgb(100, 200, 120),
            ToastKind::Warning => Color32::from_rgb(230, 180, 60),
            ToastKind::Error => Color32::from_rgb(220, 90, 90),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
    pub created: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            created: Instant::now(),
            duration: match kind {
                ToastKind::Info => Duration::from_secs(3),
                ToastKind::Success => Duration::from_secs(4),
                ToastKind::Warning => Duration::from_secs(5),
                ToastKind::Error => Duration::from_secs(7),
            },
        }
    }

    pub fn expires_at(&self) -> Instant {
        self.created + self.duration
    }

    pub fn age(&self) -> Duration {
        self.created.elapsed()
    }

    /// 0.0 = fully shown, 1.0 = fully hidden
    pub fn fade(&self) -> f32 {
        let total = self.duration.as_secs_f32().max(0.001);
        let a = self.age().as_secs_f32();
        if a < total * 0.85 {
            0.0
        } else {
            ((a - total * 0.85) / (total * 0.15)).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Default)]
pub struct ToastQueue {
    pub items: Vec<Toast>,
    pub max_visible: usize,
}

impl ToastQueue {
    pub fn new() -> Self {
        Self { items: Vec::new(), max_visible: 5 }
    }

    pub fn push(&mut self, kind: ToastKind, message: impl Into<String>) {
        self.items.push(Toast::new(kind, message));
        if self.items.len() > 32 {
            self.items.remove(0);
        }
    }

    pub fn info(&mut self, m: impl Into<String>) { self.push(ToastKind::Info, m); }
    pub fn success(&mut self, m: impl Into<String>) { self.push(ToastKind::Success, m); }
    pub fn warn(&mut self, m: impl Into<String>) { self.push(ToastKind::Warning, m); }
    pub fn error(&mut self, m: impl Into<String>) { self.push(ToastKind::Error, m); }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.items.retain(|t| t.expires_at() > now);
    }

    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

pub fn render(ctx: &Context, queue: &ToastQueue) {
    if queue.items.is_empty() { return; }

    let screen = ctx.screen_rect();
    let pad = 16.0;
    let toast_w = 320.0;
    let toast_h = 52.0;
    let gap = 8.0;

    let n = queue.items.len().min(queue.max_visible);
    for (i, t) in queue.items.iter().rev().take(n).enumerate() {
        let stack_idx = i as f32;
        let fade = t.fade();
        let alpha = (1.0 - fade) * (1.0 - stack_idx * 0.05).max(0.5);

        let color = t.kind.color();
        let fill = Color32::from_rgba_unmultiplied(35, 35, 45, (220.0 * alpha) as u8);
        let stroke_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (200.0 * alpha) as u8);
        let text_color = Color32::from_rgba_unmultiplied(230, 230, 240, (255.0 * alpha) as u8);

        let pos = egui::pos2(
            screen.max.x - toast_w - pad,
            screen.min.y + pad + stack_idx * (toast_h + gap),
        );
        let rect = egui::Rect::from_min_size(pos, Vec2::new(toast_w, toast_h));

        egui::Area::new(egui::Id::new(("toast", i, t.created.elapsed().as_millis())))
            .anchor(Align2::LEFT_TOP, Vec2::ZERO)
            .fixed_pos(pos)
            .interactable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let sense = Sense::click_and_drag();
                let resp = ui.allocate_rect(egui::Rect::from_min_size(ui.next_widget_position(), Vec2::new(toast_w, toast_h)), sense);

                ui.painter().rect_filled(rect, 6.0, fill);
                ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0, stroke_color), StrokeKind::Inside);

                // accent bar on the left
                let bar = egui::Rect::from_min_size(pos, Vec2::new(4.0, toast_h));
                ui.painter().rect_filled(bar, 2.0, color);

                // icon (just a colored dot in the kind's color for now)
                let dot_center = pos + Vec2::new(20.0, toast_h * 0.5);
                ui.painter().circle_filled(dot_center, 5.0, color);

                // message text
                let text_pos = pos + Vec2::new(36.0, 10.0);
                let max_text_w = toast_w - 48.0;
                let galley = ui.painter().layout(
                    t.message.clone(),
                    FontId::proportional(13.0),
                    text_color,
                    max_text_w,
                );
                ui.painter().galley(text_pos, galley, text_color);

                // close button if hovered
                if resp.hovered() {
                    let close_pos = pos + Vec2::new(toast_w - 18.0, toast_h * 0.5);
                    ui.painter().text(close_pos, Align2::CENTER_CENTER, "×", FontId::proportional(16.0), text_color);
                }
            });
    }
}
