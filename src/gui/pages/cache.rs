use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Cache Management");
    ui.add_space(8.0);

    let zone_id = match state.zone_id() {
        Some(id) => id,
        None => {
            ui.label("Please select a zone first.");
            return;
        }
    };

    if ui.button("\u{1F504} Refresh").clicked() {
        load_cache_status(state, ctx, &zone_id);
    }
    ui.add_space(8.0);

    // Cache Level
    ui.group(|ui| {
        ui.label(egui::RichText::new("Cache Level").strong());
        ui.horizontal(|ui| {
            for level in &["aggressive", "basic", "simplified"] {
                let selected = state.cache_level == *level;
                if ui.selectable_label(selected, *level).clicked() && !selected {
                    set_cache_level(state, ctx, &zone_id, level);
                }
            }
        });
    });
    ui.add_space(8.0);

    // Browser Cache TTL
    ui.group(|ui| {
        ui.label(egui::RichText::new("Browser Cache TTL").strong());
        ui.horizontal(|ui| {
            let mut ttl = state.browser_cache_ttl as f32;
            ui.label(format!("{} seconds", state.browser_cache_ttl));
            if ui.add(egui::Slider::new(&mut ttl, 0.0..=86400.0).text("seconds")).changed() {
                state.browser_cache_ttl = ttl as u32;
            }
            if ui.button("Apply").clicked() {
                set_browser_ttl(state, ctx, &zone_id, state.browser_cache_ttl);
            }
        });
    });
    ui.add_space(8.0);

    // Development Mode
    ui.group(|ui| {
        ui.label(egui::RichText::new("Development Mode").strong());
        ui.horizontal(|ui| {
            let mut dev = state.dev_mode_on;
            if ui.checkbox(&mut dev, "Enable Development Mode").changed() {
                toggle_dev_mode(state, ctx, &zone_id, dev);
            }
            if state.dev_mode_on {
                ui.label(egui::RichText::new("(Cache bypassed for 3 hours)").color(theme::WARNING).small());
            }
        });
    });
    ui.add_space(8.0);

    // Purge Cache
    ui.group(|ui| {
        ui.label(egui::RichText::new("Purge Cache").strong());
        if ui.button(egui::RichText::new("Purge Everything").color(theme::DANGER)).clicked() {
            state.confirm_dialog = Some(ConfirmDialog {
                title: "Purge All Cache".to_string(),
                message: "This will purge ALL cached files. Continue?".to_string(),
                action: ConfirmAction::PurgeAllCache(zone_id.clone()),
            });
        }
        ui.add_space(4.0);
        ui.label("Purge by URLs (one per line):");
        ui.add(
            egui::TextEdit::multiline(&mut state.purge_urls_input)
                .desired_width(f32::INFINITY)
                .desired_rows(4),
        );
        if ui.button("Purge URLs").clicked() && !state.purge_urls_input.is_empty() {
            purge_by_urls(state, ctx, &zone_id);
        }
    });
}

pub fn load_cache_status(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client {
        Some(c) => c.clone(),
        None => return,
    };
    let zid = zone_id.to_string();
    state.set_loading("Loading cache status...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let level = client.get_cache_level(&zid).await;
        let ttl = client.get_browser_cache_ttl(&zid).await;
        match (level, ttl) {
            (Ok(l), Ok(t)) => {
                AsyncResult::CacheStatusLoaded(Ok((l, t, false)))
            }
            (Err(e), _) => AsyncResult::CacheStatusLoaded(Err(e)),
            (_, Err(e)) => AsyncResult::CacheStatusLoaded(Err(e)),
        }
    });
}

fn set_cache_level(state: &mut AppState, ctx: &egui::Context, zone_id: &str, level: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    let l = level.to_string();
    state.set_loading("Setting cache level...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_cache_level(&zid, &l).await;
        AsyncResult::CacheActionDone(result.map(|_| format!("Cache level set to {}", l)))
    });
}

fn set_browser_ttl(state: &mut AppState, ctx: &egui::Context, zone_id: &str, ttl: u32) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    state.set_loading("Setting browser TTL...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_browser_cache_ttl(&zid, ttl).await;
        AsyncResult::CacheActionDone(result.map(|_| format!("Browser TTL set to {}", ttl)))
    });
}

fn toggle_dev_mode(state: &mut AppState, ctx: &egui::Context, zone_id: &str, enable: bool) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    state.set_loading("Toggling dev mode...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.set_development_mode(&zid, enable).await;
        AsyncResult::CacheActionDone(result.map(|_| if enable { "Dev mode enabled" } else { "Dev mode disabled" }.to_string()))
    });
}

fn purge_by_urls(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let urls: Vec<String> = state.purge_urls_input.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    if urls.is_empty() { return; }
    let count = urls.len();
    let zid = zone_id.to_string();
    state.purge_urls_input.clear();
    state.set_loading("Purging URLs...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let result = client.purge_cache_by_urls(&zid, urls).await;
        AsyncResult::CachePurged(result.map(|_| format!("Purged {} URLs", count)))
    });
}
