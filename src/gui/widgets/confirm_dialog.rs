use eframe::egui;
use super::super::state::{AppState, ConfirmAction, NotifLevel};
use super::super::async_bridge::spawn_async;
use super::super::state::AsyncResult;

pub fn render_confirm_dialog(state: &mut AppState, ctx: &egui::Context) {
    let dialog = match &state.confirm_dialog {
        Some(d) => d,
        None => return,
    };

    let title = dialog.title.clone();
    let message = dialog.message.clone();
    let mut close = false;
    let mut confirmed = false;

    egui::Window::new(&title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(&message);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                if ui
                    .button(egui::RichText::new("Confirm").color(super::super::theme::DANGER))
                    .clicked()
                {
                    confirmed = true;
                    close = true;
                }
            });
        });

    if confirmed {
        if let Some(dialog) = state.confirm_dialog.take() {
            execute_confirm_action(state, ctx, dialog.action);
        }
    } else if close {
        state.confirm_dialog = None;
    }
}

fn execute_confirm_action(state: &mut AppState, ctx: &egui::Context, action: ConfirmAction) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => {
            state.notify("No client configured", NotifLevel::Error);
            return;
        }
    };

    match action {
        ConfirmAction::DeleteZone(zone_id) => {
            state.set_loading("Deleting zone...");
            let zid = zone_id.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.delete_zone(&zid).await;
                AsyncResult::ZoneDeleted(result.map(|_| zid))
            });
        }
        ConfirmAction::DeleteDnsRecord(zone_id, record_id) => {
            state.set_loading("Deleting DNS record...");
            let zid = zone_id.clone();
            let rid = record_id.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.delete_dns_record(&zid, &rid).await;
                AsyncResult::DnsRecordDeleted(result.map(|_| rid))
            });
        }
        ConfirmAction::DeletePageRule(zone_id, rule_id) => {
            state.set_loading("Deleting page rule...");
            let zid = zone_id.clone();
            let rid = rule_id.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.delete_page_rule(&zid, &rid).await;
                AsyncResult::PageRuleDeleted(result.map(|_| rid))
            });
        }
        ConfirmAction::DeleteWorker(name) => {
            state.set_loading("Deleting worker...");
            let account_id = state.config.cloudflare.account_id.clone().unwrap_or_default();
            let n = name.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.delete_worker(&account_id, &n).await;
                AsyncResult::WorkerDeleted(result.map(|_| n))
            });
        }
        ConfirmAction::PurgeAllCache(zone_id) => {
            state.set_loading("Purging all cache...");
            let zid = zone_id.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.purge_all_cache(&zid).await;
                AsyncResult::CachePurged(result.map(|_| "All cache purged".to_string()))
            });
        }
        ConfirmAction::DeleteIpRule(zone_id, rule_id) => {
            state.set_loading("Deleting IP rule...");
            let zid = zone_id.clone();
            let rid = rule_id.clone();
            spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
                let result = client.delete_ip_access_rule(&zid, &rid).await;
                AsyncResult::IpRuleDeleted(result.map(|_| rid))
            });
        }
    }
}
