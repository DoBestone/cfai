use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Page Rules");
    ui.add_space(8.0);

    let zone_id = match state.zone_id() {
        Some(id) => id,
        None => {
            ui.label("Please select a zone first.");
            return;
        }
    };

    if ui.button("\u{1F504} Refresh").clicked() {
        load_page_rules(state, ctx, &zone_id);
    }
    ui.add_space(8.0);

    // Create redirect form
    ui.group(|ui| {
        ui.label(egui::RichText::new("Create Redirect Rule").strong());
        ui.horizontal(|ui| {
            ui.label("URL Pattern:");
            ui.text_edit_singleline(&mut state.redirect_form.url_pattern);
        });
        ui.horizontal(|ui| {
            ui.label("Redirect To:");
            ui.text_edit_singleline(&mut state.redirect_form.redirect_url);
            ui.label("Status:");
            egui::ComboBox::from_id_salt("redirect_status")
                .selected_text(state.redirect_form.status_code.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.redirect_form.status_code, 301, "301 Permanent");
                    ui.selectable_value(&mut state.redirect_form.status_code, 302, "302 Temporary");
                });
            if ui.button("Create").clicked() {
                create_redirect(state, ctx, &zone_id);
            }
        });
    });
    ui.add_space(8.0);

    // Page rules table
    if state.page_rules.is_empty() {
        ui.label("No page rules.");
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("page_rules_table")
                .num_columns(5)
                .striped(true)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("URL Pattern");
                    ui.strong("Actions");
                    ui.strong("Priority");
                    ui.strong("Status");
                    ui.strong("Delete");
                    ui.end_row();

                    for rule in state.page_rules.clone() {
                        let pattern = rule
                            .targets
                            .as_ref()
                            .and_then(|t| t.first())
                            .and_then(|t| t.constraint.as_ref())
                            .and_then(|c| c.value.clone())
                            .unwrap_or_else(|| "-".to_string());
                        ui.label(&pattern);

                        let actions_str = rule
                            .actions
                            .as_ref()
                            .map(|acts| {
                                acts.iter()
                                    .map(|a| a.id.as_deref().unwrap_or("?"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_else(|| "-".to_string());
                        ui.label(egui::RichText::new(actions_str).small());

                        ui.label(rule.priority.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()));

                        let status = rule.status.as_deref().unwrap_or("-");
                        let sc = if status == "active" { theme::SUCCESS } else { theme::WARNING };
                        ui.label(egui::RichText::new(status).color(sc));

                        if let Some(id) = &rule.id {
                            if ui.small_button(egui::RichText::new("Delete").color(theme::DANGER)).clicked() {
                                state.confirm_dialog = Some(ConfirmDialog {
                                    title: "Delete Page Rule".to_string(),
                                    message: format!("Delete page rule for '{}'?", pattern),
                                    action: ConfirmAction::DeletePageRule(zone_id.clone(), id.clone()),
                                });
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

pub fn load_page_rules(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    state.set_loading("Loading page rules...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.list_page_rules(&zid).await;
        AsyncResult::PageRulesLoaded(result)
    });
}

fn create_redirect(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let pattern = state.redirect_form.url_pattern.trim().to_string();
    let target = state.redirect_form.redirect_url.trim().to_string();
    let status = state.redirect_form.status_code;
    if pattern.is_empty() || target.is_empty() { return; }
    let zid = zone_id.to_string();
    state.redirect_form = RedirectForm::default();
    state.set_loading("Creating redirect...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.create_redirect_rule(&zid, &pattern, &target, status).await;
        AsyncResult::PageRuleCreated(result.map(|_| format!("Redirect created: {} -> {}", pattern, target)))
    });
}
