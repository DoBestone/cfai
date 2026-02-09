use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;
use crate::models::zone::{CreateZoneRequest, ZoneListParams};

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Zone Management");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("\u{1F504} Refresh").clicked() {
            load_zones(state, ctx);
        }
        ui.separator();
        ui.label("Search:");
        ui.text_edit_singleline(&mut state.zone_search);
    });
    ui.add_space(4.0);

    // Add zone form
    ui.collapsing("Add Zone", |ui| {
        ui.horizontal(|ui| {
            ui.label("Domain:");
            ui.text_edit_singleline(&mut state.zone_add_domain);
            if ui.button("Add").clicked() && !state.zone_add_domain.is_empty() {
                add_zone(state, ctx);
            }
        });
    });
    ui.add_space(8.0);

    // Zone table
    let search = state.zone_search.to_lowercase();
    let filtered: Vec<_> = state
        .zones
        .iter()
        .filter(|z| search.is_empty() || z.name.to_lowercase().contains(&search))
        .cloned()
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("zone_table")
            .num_columns(6)
            .striped(true)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                // Header
                ui.strong("Name");
                ui.strong("Status");
                ui.strong("Plan");
                ui.strong("Paused");
                ui.strong("Nameservers");
                ui.strong("Actions");
                ui.end_row();

                for zone in &filtered {
                    ui.label(egui::RichText::new(&zone.name).color(theme::ACCENT));

                    let sc = match zone.status.as_str() {
                        "active" => theme::SUCCESS,
                        "pending" => theme::WARNING,
                        _ => theme::DANGER,
                    };
                    ui.label(egui::RichText::new(&zone.status).color(sc));

                    let plan_name = zone
                        .plan
                        .as_ref()
                        .and_then(|p| p.name.clone())
                        .unwrap_or_else(|| "-".to_string());
                    ui.label(plan_name);

                    let paused = zone.paused.unwrap_or(false);
                    ui.label(if paused { "Yes" } else { "No" });

                    let ns = zone
                        .name_servers
                        .as_ref()
                        .map(|v| v.join(", "))
                        .unwrap_or_else(|| "-".to_string());
                    ui.label(egui::RichText::new(ns).small());

                    ui.horizontal(|ui| {
                        if ui.small_button("Select").clicked() {
                            state.selected_zone = Some(zone.clone());
                        }
                        let pause_label = if paused { "Resume" } else { "Pause" };
                        if ui.small_button(pause_label).clicked() {
                            toggle_pause(state, ctx, &zone.id, paused);
                        }
                        if ui
                            .small_button(egui::RichText::new("Delete").color(theme::DANGER))
                            .clicked()
                        {
                            state.confirm_dialog = Some(ConfirmDialog {
                                title: "Delete Zone".to_string(),
                                message: format!("Delete zone '{}'? This cannot be undone.", zone.name),
                                action: ConfirmAction::DeleteZone(zone.id.clone()),
                            });
                        }
                    });
                    ui.end_row();
                }
            });
    });

    // Zone settings
    if let Some(zone) = &state.selected_zone.clone() {
        ui.add_space(12.0);
        ui.separator();
        ui.heading(format!("Settings: {}", zone.name));
        ui.horizontal(|ui| {
            if ui.button("Load Settings").clicked() {
                load_settings(state, ctx, &zone.id);
            }
        });
        if !state.zone_settings.is_empty() {
            egui::ScrollArea::vertical()
                .id_salt("zone_settings_scroll")
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("settings_table")
                        .num_columns(3)
                        .striped(true)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            ui.strong("Setting");
                            ui.strong("Value");
                            ui.strong("Editable");
                            ui.end_row();
                            for s in &state.zone_settings {
                                ui.label(&s.id);
                                ui.label(
                                    egui::RichText::new(format!("{}", s.value))
                                        .small()
                                        .weak(),
                                );
                                let editable = s.editable.unwrap_or(false);
                                ui.label(if editable { "Yes" } else { "No" });
                                ui.end_row();
                            }
                        });
                });
        }
    }
}

fn load_zones(state: &mut AppState, ctx: &egui::Context) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    state.set_loading("Loading zones...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let params = ZoneListParams { per_page: Some(50), ..Default::default() };
        let result = client.list_zones(&params).await;
        AsyncResult::ZonesLoaded(result.map(|r| r.result.unwrap_or_default()))
    });
}

fn add_zone(state: &mut AppState, ctx: &egui::Context) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let domain = state.zone_add_domain.trim().to_string();
    let account_id = state.config.cloudflare.account_id.clone();
    state.zone_add_domain.clear();
    state.set_loading("Creating zone...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let req = CreateZoneRequest {
            name: domain,
            account: account_id.map(|id| crate::models::zone::CreateZoneAccount { id }),
            zone_type: None,
            jump_start: Some(true),
        };
        let result = client.create_zone(&req).await;
        AsyncResult::ZoneCreated(result)
    });
}

fn toggle_pause(state: &mut AppState, ctx: &egui::Context, zone_id: &str, currently_paused: bool) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Toggling zone...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.toggle_zone_pause(&zid, !currently_paused).await;
        AsyncResult::ZoneToggled(result)
    });
}

fn load_settings(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Loading settings...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.get_zone_settings(&zid).await;
        AsyncResult::ZoneSettingsLoaded(result)
    });
}
