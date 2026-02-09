use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::{AppState, AsyncResult, NotifLevel};
use crate::gui::theme;
use crate::models::zone::ZoneListParams;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Dashboard");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("\u{1F504} Refresh Zones").clicked() {
            load_zones(state, ctx);
        }
        ui.label(format!("{} zones loaded", state.zones.len()));
    });
    ui.add_space(8.0);

    if state.zones.is_empty() && !state.loading {
        ui.label("No zones loaded. Click Refresh or check your configuration.");
        return;
    }

    // Zone cards grid
    let available_width = ui.available_width();
    let card_width = 300.0_f32;
    let cols = ((available_width / card_width) as usize).max(1);

    egui::Grid::new("zone_grid")
        .num_columns(cols)
        .spacing([12.0, 12.0])
        .show(ui, |ui| {
            for (i, zone) in state.zones.clone().iter().enumerate() {
                if i > 0 && i % cols == 0 {
                    ui.end_row();
                }
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(31, 41, 55))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 65, 81)))
                    .show(ui, |ui| {
                        ui.set_min_width(280.0);
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new(&zone.name)
                                    .color(theme::ACCENT)
                                    .size(16.0),
                            );
                        });
                        ui.add_space(4.0);

                        let status_color = match zone.status.as_str() {
                            "active" => theme::SUCCESS,
                            "pending" => theme::WARNING,
                            _ => theme::DANGER,
                        };
                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            ui.label(
                                egui::RichText::new(&zone.status)
                                    .color(status_color)
                                    .strong(),
                            );
                        });

                        if let Some(plan) = &zone.plan {
                            if let Some(name) = &plan.name {
                                ui.horizontal(|ui| {
                                    ui.label("Plan:");
                                    ui.label(name);
                                });
                            }
                        }

                        let paused = zone.paused.unwrap_or(false);
                        ui.horizontal(|ui| {
                            ui.label("Paused:");
                            ui.label(if paused { "Yes" } else { "No" });
                        });

                        if let Some(ns) = &zone.name_servers {
                            if !ns.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("NS:");
                                    ui.label(
                                        egui::RichText::new(ns.join(", ")).small().weak(),
                                    );
                                });
                            }
                        }

                        ui.add_space(4.0);
                        if ui
                            .button(egui::RichText::new("Select").color(theme::ACCENT))
                            .clicked()
                        {
                            state.selected_zone = Some(zone.clone());
                        }
                    });
            }
        });
}

pub fn load_zones(state: &mut AppState, ctx: &egui::Context) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => {
            state.notify("No client configured", NotifLevel::Error);
            return;
        }
    };
    state.set_loading("Loading zones...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let params = ZoneListParams {
            per_page: Some(50),
            ..Default::default()
        };
        let result = client.list_zones(&params).await;
        AsyncResult::ZonesLoaded(result.map(|r| r.result.unwrap_or_default()))
    });
}
