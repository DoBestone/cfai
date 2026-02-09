use eframe::egui;
use super::super::state::AppState;

pub fn render_status_bar(state: &AppState, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(zone) = &state.selected_zone {
                ui.label(
                    egui::RichText::new(format!("Zone: {}", zone.name))
                        .small()
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Status: {}", zone.status))
                        .small(),
                );
            } else {
                ui.label(egui::RichText::new("No zone selected").small().weak());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("CFAI v0.3.8").small().weak());
            });
        });
    });
}
