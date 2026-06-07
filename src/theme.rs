use catppuccin_egui::{FRAPPE, LATTE, MACCHIATO, MOCHA};
use eframe::egui::{self, Color32, Context, CornerRadius, FontFamily, FontId, Margin, Stroke, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Mocha,
    Macchiato,
    Frappe,
    Latte,
}

impl ThemeChoice {
    pub fn display(self) -> &'static str {
        match self {
            ThemeChoice::Mocha => "Mocha (dark)",
            ThemeChoice::Macchiato => "Macchiato (dark)",
            ThemeChoice::Frappe => "Frappé (dark)",
            ThemeChoice::Latte => "Latte (light)",
        }
    }

    pub fn all() -> [ThemeChoice; 4] {
        [ThemeChoice::Mocha, ThemeChoice::Macchiato, ThemeChoice::Frappe, ThemeChoice::Latte]
    }

    pub fn is_dark(self) -> bool {
        !matches!(self, ThemeChoice::Latte)
    }
}

/// Apply a Catppuccin palette and our custom polish layer.
pub fn apply(ctx: &Context, choice: ThemeChoice) {
    let palette = match choice {
        ThemeChoice::Mocha => MOCHA,
        ThemeChoice::Macchiato => MACCHIATO,
        ThemeChoice::Frappe => FRAPPE,
        ThemeChoice::Latte => LATTE,
    };

    // catppuccin_egui::set_theme modifies the Context's default style
    // for both the light and dark theme.
    catppuccin_egui::set_theme(ctx, palette);

    let theme = if choice.is_dark() { Theme::Dark } else { Theme::Light };
    ctx.style_mut_of(theme, |style| {
        polish(style, choice.is_dark());
    });
}

fn polish(style: &mut egui::Style, dark: bool) {
    // Round corners
    style.visuals.window_corner_radius = CornerRadius::same(10);
    style.visuals.menu_corner_radius = CornerRadius::same(8);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.open.corner_radius = CornerRadius::same(8);

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = Margin::same(12);

    // Strokes
    let border = if dark {
        Color32::from_rgba_unmultiplied(127, 127, 127, 60)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 40)
    };
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);

    // Typography
    for (text_style, size) in [
        (egui::TextStyle::Heading, 22.0_f32),
        (egui::TextStyle::Body, 14.0),
        (egui::TextStyle::Button, 14.0),
        (egui::TextStyle::Small, 12.0),
        (egui::TextStyle::Monospace, 13.0),
    ] {
        if let Some(f) = style.text_styles.get_mut(&text_style) {
            f.size = size;
        }
    }

    // Selection colors
    if dark {
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(137, 180, 250, 90);
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(137, 180, 250));
    } else {
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(30, 102, 197, 40);
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(30, 102, 197));
    }
}
