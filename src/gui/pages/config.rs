use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            save_config(state, ctx);
        }
        if ui.button("Verify Token").clicked() {
            verify_token(state, ctx);
        }
        ui.checkbox(&mut state.config_show_secrets, "Show Secrets");
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Cloudflare section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Cloudflare API").strong().color(theme::ACCENT));
            ui.add_space(4.0);

            config_field(ui, "API Token", &mut state.config_edit.cloudflare.api_token, state.config_show_secrets);
            config_field(ui, "Email", &mut state.config_edit.cloudflare.email, true);
            config_field(ui, "API Key", &mut state.config_edit.cloudflare.api_key, state.config_show_secrets);
            config_field(ui, "Account ID", &mut state.config_edit.cloudflare.account_id, true);
        });
        ui.add_space(8.0);

        // AI section
        ui.group(|ui| {
            ui.label(egui::RichText::new("AI Configuration").strong().color(theme::ACCENT));
            ui.add_space(4.0);

            config_field(ui, "API URL", &mut state.config_edit.ai.api_url, true);
            config_field(ui, "API Key", &mut state.config_edit.ai.api_key, state.config_show_secrets);
            config_field(ui, "Model", &mut state.config_edit.ai.model, true);

            ui.horizontal(|ui| {
                ui.label("Max Tokens:");
                let mut val = state.config_edit.ai.max_tokens.unwrap_or(4096).to_string();
                if ui.add(egui::TextEdit::singleline(&mut val).desired_width(80.0)).changed() {
                    state.config_edit.ai.max_tokens = val.parse().ok();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Temperature:");
                let mut temp = state.config_edit.ai.temperature.unwrap_or(0.7);
                if ui.add(egui::Slider::new(&mut temp, 0.0..=2.0)).changed() {
                    state.config_edit.ai.temperature = Some(temp);
                }
            });
        });
        ui.add_space(8.0);

        // Defaults section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Defaults").strong().color(theme::ACCENT));
            ui.add_space(4.0);

            config_field(ui, "Default Domain", &mut state.config_edit.defaults.domain, true);

            ui.horizontal(|ui| {
                ui.label("Output Format:");
                let current = state.config_edit.defaults.output_format.clone().unwrap_or_else(|| "table".to_string());
                egui::ComboBox::from_id_salt("output_format")
                    .selected_text(&current)
                    .show_ui(ui, |ui| {
                        for fmt in &["table", "json", "plain"] {
                            if ui.selectable_label(current == *fmt, *fmt).clicked() {
                                state.config_edit.defaults.output_format = Some(fmt.to_string());
                            }
                        }
                    });
            });
        });

        ui.add_space(12.0);
        let path = crate::config::settings::AppConfig::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        ui.label(egui::RichText::new(format!("Config file: {}", path)).small().weak());
    });
}

fn config_field(ui: &mut egui::Ui, label: &str, value: &mut Option<String>, show: bool) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        let mut display = value.clone().unwrap_or_default();
        if !show && !display.is_empty() {
            let masked = "*".repeat(display.len().min(20));
            ui.label(masked);
        } else {
            if ui.text_edit_singleline(&mut display).changed() {
                *value = if display.is_empty() { None } else { Some(display) };
            }
        }
    });
}

fn save_config(state: &mut AppState, _ctx: &egui::Context) {
    let config = state.config_edit.clone();
    match config.save() {
        Ok(()) => {
            state.config = config;
            state.notify("Configuration saved", NotifLevel::Success);
        }
        Err(e) => {
            state.notify(format!("Save failed: {}", e), NotifLevel::Error);
        }
    }
}

fn verify_token(state: &mut AppState, ctx: &egui::Context) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => {
            state.notify("No client configured", NotifLevel::Error);
            return;
        }
    };
    state.set_loading("Verifying token...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.verify_token().await;
        AsyncResult::TokenVerified(result)
    });
}
