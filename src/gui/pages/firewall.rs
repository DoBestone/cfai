use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Firewall Management");
    ui.add_space(8.0);

    let zone_id = match state.zone_id() {
        Some(id) => id,
        None => {
            ui.label("Please select a zone first.");
            return;
        }
    };

    if ui.button("\u{1F504} Refresh").clicked() {
        load_firewall(state, ctx, &zone_id);
    }
    ui.add_space(8.0);

    // Security Level
    ui.group(|ui| {
        ui.label(egui::RichText::new("Security Level").strong());
        ui.horizontal(|ui| {
            for level in &["off", "essentially_off", "low", "medium", "high", "under_attack"] {
                let selected = state.security_level == *level;
                let color = if *level == "under_attack" { theme::DANGER } else { egui::Color32::WHITE };
                if ui.selectable_label(selected, egui::RichText::new(*level).color(color)).clicked() && !selected {
                    set_security_level(state, ctx, &zone_id, level);
                }
            }
        });
    });
    ui.add_space(8.0);

    // Quick actions
    ui.group(|ui| {
        ui.label(egui::RichText::new("Quick Actions").strong());
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("Enable Under Attack Mode").color(theme::DANGER)).clicked() {
                set_under_attack(state, ctx, &zone_id, true);
            }
            if ui.button("Disable Under Attack Mode").clicked() {
                set_under_attack(state, ctx, &zone_id, false);
            }
        });
    });
    ui.add_space(8.0);

    // Block/Whitelist IP
    ui.group(|ui| {
        ui.label(egui::RichText::new("IP Access Control").strong());
        ui.horizontal(|ui| {
            ui.label("IP:");
            ui.add(egui::TextEdit::singleline(&mut state.fw_ip_input).desired_width(150.0));
            ui.label("Note:");
            ui.add(egui::TextEdit::singleline(&mut state.fw_note_input).desired_width(150.0));
            if ui.button(egui::RichText::new("Block").color(theme::DANGER)).clicked() {
                block_ip(state, ctx, &zone_id);
            }
            if ui.button(egui::RichText::new("Whitelist").color(theme::SUCCESS)).clicked() {
                whitelist_ip(state, ctx, &zone_id);
            }
        });
    });
    ui.add_space(8.0);

    // IP Access Rules table
    ui.label(egui::RichText::new("IP Access Rules").strong());
    egui::ScrollArea::vertical().id_salt("ip_rules").max_height(200.0).show(ui, |ui| {
        egui::Grid::new("ip_rules_table")
            .num_columns(5)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("IP");
                ui.strong("Mode");
                ui.strong("Notes");
                ui.strong("Created");
                ui.strong("Actions");
                ui.end_row();

                for rule in state.ip_access_rules.clone() {
                    let ip = rule.configuration.as_ref().and_then(|c| c.value.clone()).unwrap_or_default();
                    ui.label(&ip);
                    let mode = rule.mode.as_deref().unwrap_or("-");
                    let mc = match mode {
                        "block" => theme::DANGER,
                        "whitelist" => theme::SUCCESS,
                        _ => theme::WARNING,
                    };
                    ui.label(egui::RichText::new(mode).color(mc));
                    ui.label(rule.notes.as_deref().unwrap_or("-"));
                    ui.label(egui::RichText::new(rule.created_on.as_deref().unwrap_or("-")).small());
                    if let Some(id) = &rule.id {
                        if ui.small_button(egui::RichText::new("Delete").color(theme::DANGER)).clicked() {
                            state.confirm_dialog = Some(ConfirmDialog {
                                title: "Delete IP Rule".to_string(),
                                message: format!("Delete {} rule for {}?", mode, ip),
                                action: ConfirmAction::DeleteIpRule(zone_id.clone(), id.clone()),
                            });
                        }
                    }
                    ui.end_row();
                }
            });
    });
    ui.add_space(8.0);

    // Firewall Rules
    ui.label(egui::RichText::new("Firewall Rules").strong());
    egui::ScrollArea::vertical().id_salt("fw_rules").max_height(200.0).show(ui, |ui| {
        egui::Grid::new("fw_rules_table")
            .num_columns(4)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Description");
                ui.strong("Action");
                ui.strong("Priority");
                ui.strong("Paused");
                ui.end_row();

                for rule in &state.firewall_rules {
                    ui.label(rule.description.as_deref().unwrap_or("-"));
                    ui.label(rule.action.as_deref().unwrap_or("-"));
                    ui.label(rule.priority.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()));
                    ui.label(if rule.paused.unwrap_or(false) { "Yes" } else { "No" });
                    ui.end_row();
                }
            });
    });
}

pub fn load_firewall(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    let zid2 = zid.clone();
    let zid3 = zid.clone();
    let c2 = client.clone();
    let c3 = client.clone();
    state.set_loading("Loading firewall...");

    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.list_firewall_rules(&zid).await;
        AsyncResult::FirewallRulesLoaded(result)
    });
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = c2.list_ip_access_rules(&zid2).await;
        AsyncResult::IpAccessRulesLoaded(result)
    });
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = c3.get_security_level(&zid3).await;
        AsyncResult::SecurityLevelLoaded(result)
    });
}

fn set_security_level(state: &mut AppState, ctx: &egui::Context, zone_id: &str, level: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    let l = level.to_string();
    state.set_loading("Setting security level...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_security_level(&zid, &l).await;
        AsyncResult::FirewallActionDone(result.map(|_| format!("Security level set to {}", l)))
    });
}

fn set_under_attack(state: &mut AppState, ctx: &egui::Context, zone_id: &str, enable: bool) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    state.set_loading("Setting Under Attack mode...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_under_attack_mode(&zid, enable).await;
        AsyncResult::FirewallActionDone(result.map(|_| if enable { "Under Attack enabled" } else { "Under Attack disabled" }.to_string()))
    });
}

fn block_ip(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let ip = state.fw_ip_input.trim().to_string();
    let note = state.fw_note_input.trim().to_string();
    if ip.is_empty() { return; }
    let zid = zone_id.to_string();
    state.fw_ip_input.clear();
    state.fw_note_input.clear();
    state.set_loading("Blocking IP...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let note_opt = if note.is_empty() { None } else { Some(note.as_str()) };
        let result = client.block_ip(&zid, &ip, note_opt).await;
        AsyncResult::IpRuleCreated(result.map(|_| format!("Blocked {}", ip)))
    });
}

fn whitelist_ip(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let ip = state.fw_ip_input.trim().to_string();
    let note = state.fw_note_input.trim().to_string();
    if ip.is_empty() { return; }
    let zid = zone_id.to_string();
    state.fw_ip_input.clear();
    state.fw_note_input.clear();
    state.set_loading("Whitelisting IP...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let note_opt = if note.is_empty() { None } else { Some(note.as_str()) };
        let result = client.whitelist_ip(&zid, &ip, note_opt).await;
        AsyncResult::IpRuleCreated(result.map(|_| format!("Whitelisted {}", ip)))
    });
}
