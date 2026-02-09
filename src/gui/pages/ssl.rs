use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("SSL/TLS Management");
    ui.add_space(8.0);

    let zone_id = match state.zone_id() {
        Some(id) => id,
        None => {
            ui.label("Please select a zone first.");
            return;
        }
    };

    if ui.button("\u{1F504} Refresh").clicked() {
        load_ssl_status(state, ctx, &zone_id);
        load_ssl_certs(state, ctx, &zone_id);
    }
    ui.add_space(8.0);

    // SSL Mode
    ui.group(|ui| {
        ui.label(egui::RichText::new("SSL/TLS Mode").strong());
        ui.horizontal(|ui| {
            for mode in &["off", "flexible", "full", "strict"] {
                let selected = state.ssl_mode == *mode;
                let label = match *mode {
                    "off" => "Off",
                    "flexible" => "Flexible",
                    "full" => "Full",
                    "strict" => "Full (Strict)",
                    _ => mode,
                };
                if ui.selectable_label(selected, label).clicked() && !selected {
                    set_ssl_mode(state, ctx, &zone_id, mode);
                }
            }
        });
    });
    ui.add_space(8.0);

    // Toggles
    ui.group(|ui| {
        ui.label(egui::RichText::new("HTTPS Settings").strong());
        ui.horizontal(|ui| {
            let mut https = state.ssl_always_https;
            if ui.checkbox(&mut https, "Always Use HTTPS").changed() {
                toggle_always_https(state, ctx, &zone_id, https);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Minimum TLS Version:");
            let current = state.ssl_min_tls.clone();
            egui::ComboBox::from_id_salt("min_tls")
                .selected_text(&current)
                .show_ui(ui, |ui| {
                    for v in &["1.0", "1.1", "1.2", "1.3"] {
                        if ui.selectable_label(current == *v, *v).clicked() {
                            set_min_tls(state, ctx, &zone_id, v);
                        }
                    }
                });
        });
    });
    ui.add_space(8.0);

    // Certificates
    ui.label(egui::RichText::new("SSL Certificates").strong());
    if state.ssl_certificates.is_empty() {
        ui.label("No certificates loaded.");
    } else {
        egui::Grid::new("ssl_certs")
            .num_columns(5)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Hosts");
                ui.strong("Issuer");
                ui.strong("Status");
                ui.strong("Expires");
                ui.strong("Priority");
                ui.end_row();

                for cert in &state.ssl_certificates {
                    let hosts = cert.hosts.as_ref().map(|h| h.join(", ")).unwrap_or_default();
                    ui.label(egui::RichText::new(hosts).small());
                    ui.label(cert.issuer.as_deref().unwrap_or("-"));
                    let status = cert.status.as_deref().unwrap_or("-");
                    let sc = if status == "active" { theme::SUCCESS } else { theme::WARNING };
                    ui.label(egui::RichText::new(status).color(sc));
                    ui.label(cert.expires_on.as_deref().unwrap_or("-"));
                    ui.label(cert.priority.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()));
                    ui.end_row();
                }
            });
    }
}

pub fn load_ssl_status(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Loading SSL status...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let mode = client.get_ssl_mode(&zid).await;
        let https = client.get_always_https(&zid).await;
        let min_tls_result = client.get_zone_setting(&zid, "min_tls_version").await;
        let min_tls = min_tls_result
            .ok()
            .map(|s| s.value.as_str().unwrap_or("1.0").to_string())
            .unwrap_or_else(|| "1.0".to_string());
        match (mode, https) {
            (Ok(m), Ok(h)) => {
                AsyncResult::SslStatusLoaded(Ok((m, h, min_tls)))
            }
            (Err(e), _) => AsyncResult::SslStatusLoaded(Err(e)),
            (_, Err(e)) => AsyncResult::SslStatusLoaded(Err(e)),
        }
    });
}

fn load_ssl_certs(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.list_ssl_certificates(&zid).await;
        AsyncResult::SslCertificatesLoaded(result)
    });
}

fn set_ssl_mode(state: &mut AppState, ctx: &egui::Context, zone_id: &str, mode: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    let m = mode.to_string();
    state.set_loading("Setting SSL mode...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_ssl_mode(&zid, &m).await;
        AsyncResult::SslModeSet(result.map(|_| m))
    });
}

fn toggle_always_https(state: &mut AppState, ctx: &egui::Context, zone_id: &str, enable: bool) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Toggling HTTPS...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_always_https(&zid, enable).await;
        AsyncResult::SslToggled(result.map(|_| if enable { "HTTPS enabled" } else { "HTTPS disabled" }.to_string()))
    });
}

fn set_min_tls(state: &mut AppState, ctx: &egui::Context, zone_id: &str, version: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    let v = version.to_string();
    state.set_loading("Setting min TLS...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_ssl_min_tls(&zid, &v).await;
        AsyncResult::SslToggled(result.map(|_| format!("Min TLS set to {}", v)))
    });
}
