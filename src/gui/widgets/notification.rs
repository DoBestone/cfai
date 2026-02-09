use eframe::egui;
use super::super::state::{AppState, NotifLevel};
use super::super::theme;

pub fn render_notifications(state: &mut AppState, ctx: &egui::Context) {
    // Remove expired
    state.notifications.retain(|n| !n.is_expired());

    if state.notifications.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("notifications"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            for notif in &state.notifications {
                let (color, prefix) = match notif.level {
                    NotifLevel::Success => (theme::SUCCESS, "\u{2705}"),
                    NotifLevel::Error => (theme::DANGER, "\u{274C}"),
                    NotifLevel::Warning => (theme::WARNING, "\u{26A0}\u{FE0F}"),
                    NotifLevel::Info => (theme::INFO, "\u{2139}\u{FE0F}"),
                };
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(31, 41, 55))
                    .stroke(egui::Stroke::new(1.0, color))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_max_width(350.0);
                        ui.label(
                            egui::RichText::new(format!("{} {}", prefix, notif.message))
                                .color(color),
                        );
                    });
                ui.add_space(4.0);
            }
        });
}
