use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;
use crate::models::dns::{DnsListParams, DnsRecordRequest};

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("DNS Management");
    ui.add_space(8.0);

    let zone_id = match state.zone_id() {
        Some(id) => id,
        None => {
            ui.label("Please select a zone first.");
            return;
        }
    };

    ui.horizontal(|ui| {
        if ui.button("\u{1F504} Refresh").clicked() {
            load_dns(state, ctx, &zone_id);
        }
        ui.separator();
        ui.label("Type:");
        egui::ComboBox::from_id_salt("dns_type_filter")
            .selected_text(if state.dns_filter_type.is_empty() { "All" } else { &state.dns_filter_type })
            .show_ui(ui, |ui| {
                if ui.selectable_label(state.dns_filter_type.is_empty(), "All").clicked() {
                    state.dns_filter_type.clear();
                }
                for t in &["A", "AAAA", "CNAME", "TXT", "MX", "NS", "SRV", "CAA"] {
                    if ui.selectable_label(state.dns_filter_type == *t, *t).clicked() {
                        state.dns_filter_type = t.to_string();
                    }
                }
            });
        ui.label("Search:");
        ui.text_edit_singleline(&mut state.dns_search);
        ui.separator();
        let add_label = if state.dns_show_add { "Cancel" } else { "+ Add Record" };
        if ui.button(add_label).clicked() {
            state.dns_show_add = !state.dns_show_add;
            if state.dns_show_add {
                state.dns_add_form = DnsAddForm::default();
            }
        }
        if ui.button("Export").clicked() {
            export_dns(state, ctx, &zone_id);
        }
    });
    ui.add_space(4.0);

    // Add record form
    if state.dns_show_add {
        render_add_form(state, ctx, ui, &zone_id);
    }

    // Edit form
    if state.dns_edit_form.is_some() {
        render_edit_form(state, ctx, ui, &zone_id);
    }

    // DNS records table
    let search = state.dns_search.to_lowercase();
    let filter_type = state.dns_filter_type.clone();
    let filtered: Vec<_> = state
        .dns_records
        .iter()
        .filter(|r| filter_type.is_empty() || r.record_type == filter_type)
        .filter(|r| search.is_empty() || r.name.to_lowercase().contains(&search) || r.content.to_lowercase().contains(&search))
        .cloned()
        .collect();

    ui.label(format!("{} records", filtered.len()));
    ui.add_space(4.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("dns_table")
            .num_columns(7)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Type");
                ui.strong("Name");
                ui.strong("Content");
                ui.strong("Proxy");
                ui.strong("TTL");
                ui.strong("Priority");
                ui.strong("Actions");
                ui.end_row();

                for record in &filtered {
                    ui.label(egui::RichText::new(&record.record_type).strong().color(theme::ACCENT));
                    ui.label(&record.name);
                    ui.label(egui::RichText::new(&record.content).small());
                    let proxied = record.proxied.unwrap_or(false);
                    ui.label(if proxied { "\u{1F7E0}" } else { "\u{26AA}" });
                    ui.label(format!("{}", record.ttl.unwrap_or(1)));
                    ui.label(record.priority.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()));

                    ui.horizontal(|ui| {
                        if ui.small_button("Edit").clicked() {
                            state.dns_edit_form = Some(DnsEditForm {
                                record_id: record.id.clone().unwrap_or_default(),
                                record_type: record.record_type.clone(),
                                name: record.name.clone(),
                                content: record.content.clone(),
                                ttl: record.ttl.unwrap_or(1).to_string(),
                                proxied: record.proxied.unwrap_or(false),
                                priority: record.priority.map(|p| p.to_string()).unwrap_or_default(),
                                comment: record.comment.clone().unwrap_or_default(),
                            });
                        }
                        if ui.small_button(egui::RichText::new("Del").color(theme::DANGER)).clicked() {
                            if let Some(id) = &record.id {
                                let zid = state.zone_id().unwrap();
                                state.confirm_dialog = Some(ConfirmDialog {
                                    title: "Delete DNS Record".to_string(),
                                    message: format!("Delete {} record '{}'?", record.record_type, record.name),
                                    action: ConfirmAction::DeleteDnsRecord(zid, id.clone()),
                                });
                            }
                        }
                    });
                    ui.end_row();
                }
            });
    });
}

fn render_add_form(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui, zone_id: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(31, 41, 55))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Add DNS Record").strong());
            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("dns_add_type")
                    .selected_text(&state.dns_add_form.record_type)
                    .show_ui(ui, |ui| {
                        for t in &["A", "AAAA", "CNAME", "TXT", "MX", "NS", "SRV", "CAA"] {
                            ui.selectable_value(&mut state.dns_add_form.record_type, t.to_string(), *t);
                        }
                    });
                ui.label("Name:");
                ui.text_edit_singleline(&mut state.dns_add_form.name);
                ui.label("Content:");
                ui.text_edit_singleline(&mut state.dns_add_form.content);
            });
            ui.horizontal(|ui| {
                ui.label("TTL:");
                ui.add(egui::TextEdit::singleline(&mut state.dns_add_form.ttl).desired_width(60.0));
                ui.checkbox(&mut state.dns_add_form.proxied, "Proxied");
                ui.label("Priority:");
                ui.add(egui::TextEdit::singleline(&mut state.dns_add_form.priority).desired_width(60.0));
                if ui.button("Create").clicked() {
                    create_dns(state, ctx, zone_id);
                }
            });
        });
    ui.add_space(4.0);
}

fn render_edit_form(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui, zone_id: &str) {
    let mut close = false;
    let mut save = false;

    if let Some(form) = &mut state.dns_edit_form {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(40, 50, 65))
            .rounding(6.0)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Edit DNS Record").strong().color(theme::WARNING));
                ui.horizontal(|ui| {
                    ui.label(format!("Type: {}", form.record_type));
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut form.name);
                    ui.label("Content:");
                    ui.text_edit_singleline(&mut form.content);
                });
                ui.horizontal(|ui| {
                    ui.label("TTL:");
                    ui.add(egui::TextEdit::singleline(&mut form.ttl).desired_width(60.0));
                    ui.checkbox(&mut form.proxied, "Proxied");
                    if ui.button("Save").clicked() {
                        save = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });
        ui.add_space(4.0);
    }

    if save {
        update_dns(state, ctx, zone_id);
    }
    if close {
        state.dns_edit_form = None;
    }
}

pub fn load_dns(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Loading DNS records...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let params = DnsListParams { per_page: Some(100), ..Default::default() };
        let result = client.list_dns_records(&zid, &params).await;
        AsyncResult::DnsRecordsLoaded(result.map(|r| r.result.unwrap_or_default()))
    });
}

fn create_dns(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let form = &state.dns_add_form;
    let req = DnsRecordRequest {
        record_type: form.record_type.clone(),
        name: form.name.clone(),
        content: form.content.clone(),
        ttl: form.ttl.parse().ok(),
        proxied: Some(form.proxied),
        priority: form.priority.parse().ok(),
        comment: if form.comment.is_empty() { None } else { Some(form.comment.clone()) },
        tags: None,
    };
    let zid = zone_id.to_string();
    state.set_loading("Creating DNS record...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.create_dns_record(&zid, &req).await;
        AsyncResult::DnsRecordCreated(result)
    });
}

fn update_dns(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let form = match &state.dns_edit_form {
        Some(f) => f,
        None => return,
    };
    let req = DnsRecordRequest {
        record_type: form.record_type.clone(),
        name: form.name.clone(),
        content: form.content.clone(),
        ttl: form.ttl.parse().ok(),
        proxied: Some(form.proxied),
        priority: form.priority.parse().ok(),
        comment: if form.comment.is_empty() { None } else { Some(form.comment.clone()) },
        tags: None,
    };
    let zid = zone_id.to_string();
    let rid = form.record_id.clone();
    state.set_loading("Updating DNS record...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.update_dns_record(&zid, &rid, &req).await;
        AsyncResult::DnsRecordUpdated(result)
    });
}

fn export_dns(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Exporting DNS...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.export_dns_records(&zid).await;
        AsyncResult::DnsExported(result)
    });
}
