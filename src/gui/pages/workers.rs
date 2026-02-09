use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Workers Management");
    ui.add_space(8.0);

    let account_id = state.config.cloudflare.account_id.clone().unwrap_or_default();
    if account_id.is_empty() {
        ui.label("Account ID not configured. Please set it in Settings.");
        return;
    }

    ui.horizontal(|ui| {
        if ui.button("\u{1F504} Refresh").clicked() {
            load_workers(state, ctx, &account_id);
        }
    });
    ui.add_space(4.0);

    // Tab bar
    ui.horizontal(|ui| {
        for (tab, label) in &[
            (WorkersTab::Scripts, "Scripts"),
            (WorkersTab::Routes, "Routes"),
            (WorkersTab::Kv, "KV Namespaces"),
            (WorkersTab::Domains, "Domains"),
        ] {
            let selected = state.workers_tab == *tab;
            if ui.selectable_label(selected, *label).clicked() {
                state.workers_tab = tab.clone();
            }
        }
    });
    ui.separator();
    ui.add_space(4.0);

    match state.workers_tab {
        WorkersTab::Scripts => render_scripts(state, ctx, ui),
        WorkersTab::Routes => render_routes(state, ui),
        WorkersTab::Kv => render_kv(state, ui),
        WorkersTab::Domains => render_domains(state, ui),
    }
}

fn render_scripts(state: &mut AppState, _ctx: &egui::Context, ui: &mut egui::Ui) {
    if state.worker_scripts.is_empty() {
        ui.label("No worker scripts.");
        return;
    }
    egui::Grid::new("workers_scripts")
        .num_columns(5)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("ID");
            ui.strong("Usage Model");
            ui.strong("Handlers");
            ui.strong("Modified");
            ui.strong("Actions");
            ui.end_row();

            for script in state.worker_scripts.clone() {
                let id = script.id.as_deref().unwrap_or("-");
                ui.label(egui::RichText::new(id).color(theme::ACCENT));
                ui.label(script.usage_model.as_deref().unwrap_or("-"));
                let handlers = script.handlers.as_ref().map(|h| h.join(", ")).unwrap_or_default();
                ui.label(handlers);
                ui.label(egui::RichText::new(script.modified_on.as_deref().unwrap_or("-")).small());
                if let Some(name) = &script.id {
                    if ui.small_button(egui::RichText::new("Delete").color(theme::DANGER)).clicked() {
                        state.confirm_dialog = Some(ConfirmDialog {
                            title: "Delete Worker".to_string(),
                            message: format!("Delete worker '{}'?", name),
                            action: ConfirmAction::DeleteWorker(name.clone()),
                        });
                    }
                }
                ui.end_row();
            }
        });
}

fn render_routes(state: &mut AppState, ui: &mut egui::Ui) {
    if state.worker_routes.is_empty() {
        ui.label("No worker routes.");
        return;
    }
    egui::Grid::new("workers_routes")
        .num_columns(3)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("Pattern");
            ui.strong("Script");
            ui.strong("ID");
            ui.end_row();

            for route in &state.worker_routes {
                ui.label(route.pattern.as_deref().unwrap_or("-"));
                ui.label(route.script.as_deref().unwrap_or("-"));
                ui.label(egui::RichText::new(route.id.as_deref().unwrap_or("-")).small().weak());
                ui.end_row();
            }
        });
}

fn render_kv(state: &mut AppState, ui: &mut egui::Ui) {
    if state.kv_namespaces.is_empty() {
        ui.label("No KV namespaces.");
        return;
    }
    egui::Grid::new("workers_kv")
        .num_columns(2)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("Title");
            ui.strong("ID");
            ui.end_row();

            for ns in &state.kv_namespaces {
                ui.label(ns.title.as_deref().unwrap_or("-"));
                ui.label(egui::RichText::new(ns.id.as_deref().unwrap_or("-")).small().weak());
                ui.end_row();
            }
        });
}

fn render_domains(state: &mut AppState, ui: &mut egui::Ui) {
    if state.worker_domains.is_empty() {
        ui.label("No worker domains.");
        return;
    }
    egui::Grid::new("workers_domains")
        .num_columns(4)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("Hostname");
            ui.strong("Service");
            ui.strong("Environment");
            ui.strong("Zone");
            ui.end_row();

            for domain in &state.worker_domains {
                ui.label(domain.hostname.as_deref().unwrap_or("-"));
                ui.label(domain.service.as_deref().unwrap_or("-"));
                ui.label(domain.environment.as_deref().unwrap_or("-"));
                ui.label(domain.zone_name.as_deref().unwrap_or("-"));
                ui.end_row();
            }
        });
}

pub fn load_workers(state: &mut AppState, ctx: &egui::Context, account_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let aid = account_id.to_string();
    let aid2 = aid.clone();
    let aid3 = aid.clone();
    let c2 = client.clone();
    let c3 = client.clone();
    state.set_loading("Loading workers...");

    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.list_workers(&aid).await;
        AsyncResult::WorkersLoaded(result)
    });
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = c2.list_kv_namespaces(&aid2).await;
        AsyncResult::KvNamespacesLoaded(result)
    });
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = c3.list_worker_domains(&aid3).await;
        AsyncResult::WorkerDomainsLoaded(result)
    });

    // Routes need zone_id
    if let Some(zone_id) = state.zone_id() {
        let c4 = state.client.as_ref().unwrap().clone();
        spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
            let result = c4.list_worker_routes(&zone_id).await;
            AsyncResult::WorkerRoutesLoaded(result)
        });
    }
}
