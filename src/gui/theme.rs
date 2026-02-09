use eframe::egui;

/// Apply dark theme with Cloudflare-orange accent
pub fn setup_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    let accent = egui::Color32::from_rgb(245, 158, 11);
    let bg_dark = egui::Color32::from_rgb(17, 24, 39);
    let bg_panel = egui::Color32::from_rgb(31, 41, 55);
    let bg_widget = egui::Color32::from_rgb(55, 65, 81);

    visuals.panel_fill = bg_dark;
    visuals.window_fill = bg_panel;
    visuals.widgets.noninteractive.bg_fill = bg_panel;
    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 85, 99);
    visuals.widgets.active.bg_fill = accent;
    visuals.selection.bg_fill = accent.linear_multiply(0.3);
    visuals.hyperlink_color = accent;
    visuals.faint_bg_color = egui::Color32::from_rgb(24, 32, 48);
    visuals.extreme_bg_color = egui::Color32::from_rgb(10, 15, 25);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(22.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(12.0),
    );
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    ctx.set_style(style);
}

/// Accent color constant
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(234, 179, 8);
pub const INFO: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
