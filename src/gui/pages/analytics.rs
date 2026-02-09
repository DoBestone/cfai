use eframe::egui;

use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;
use crate::models::analytics::AnalyticsParams;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Analytics");
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
            load_analytics(state, ctx, &zone_id);
        }
        ui.separator();
        ui.label("Period:");
        for (val, label) in &[("24h", "Last 24h"), ("7d", "Last 7 days")] {
            if ui.selectable_label(state.analytics_period == *val, *label).clicked() {
                state.analytics_period = val.to_string();
                load_analytics(state, ctx, &zone_id);
            }
        }
    });
    ui.add_space(8.0);

    let dashboard = match &state.analytics {
        Some(d) => d.clone(),
        None => {
            ui.label("No analytics data. Click Refresh to load.");
            return;
        }
    };

    if let Some(totals) = &dashboard.totals {
        // Summary cards
        ui.horizontal(|ui| {
            stat_card(ui, "Total Requests", totals.requests.as_ref().and_then(|r| r.all).unwrap_or(0), theme::ACCENT);
            stat_card(ui, "Cached", totals.requests.as_ref().and_then(|r| r.cached).unwrap_or(0), theme::SUCCESS);
            stat_card(ui, "Uncached", totals.requests.as_ref().and_then(|r| r.uncached).unwrap_or(0), theme::WARNING);
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let bw = totals.bandwidth.as_ref().and_then(|b| b.all).unwrap_or(0);
            stat_card_bytes(ui, "Bandwidth", bw, theme::INFO);
            stat_card(ui, "Threats", totals.threats.as_ref().and_then(|t| t.all).unwrap_or(0), theme::DANGER);
            stat_card(ui, "Unique Visitors", totals.uniques.as_ref().and_then(|u| u.all).unwrap_or(0), theme::ACCENT);
        });
        ui.add_space(4.0);

        // Cache hit rate
        if let Some(req) = &totals.requests {
            let all = req.all.unwrap_or(0) as f64;
            let cached = req.cached.unwrap_or(0) as f64;
            let rate = if all > 0.0 { cached / all * 100.0 } else { 0.0 };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Cache Hit Rate: {:.1}%", rate)).strong());
                let bar_width = 200.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, 16.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(55, 65, 81));
                let fill_width = (rate / 100.0) as f32 * bar_width;
                let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, 16.0));
                painter.rect_filled(fill_rect, 4.0, theme::SUCCESS);
            });
        }

        // HTTPS vs HTTP
        if let Some(req) = &totals.requests {
            if let Some(ssl) = &req.ssl {
                let enc = ssl.encrypted.unwrap_or(0);
                let unenc = ssl.unencrypted.unwrap_or(0);
                ui.horizontal(|ui| {
                    ui.label(format!("HTTPS: {} | HTTP: {}", format_number(enc), format_number(unenc)));
                });
            }
        }
    }

    ui.add_space(12.0);

    // Timeseries chart using egui_plot
    if let Some(timeseries) = &dashboard.timeseries {
        if !timeseries.is_empty() {
            ui.label(egui::RichText::new("Requests Over Time").strong());
            render_requests_chart(ui, timeseries);
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Bandwidth Over Time").strong());
            render_bandwidth_chart(ui, timeseries);
        }
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: u64, color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(31, 41, 55))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_width(150.0);
            ui.label(egui::RichText::new(label).small().weak());
            ui.label(egui::RichText::new(format_number(value)).size(20.0).color(color).strong());
        });
}

fn stat_card_bytes(ui: &mut egui::Ui, label: &str, bytes: u64, color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(31, 41, 55))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_width(150.0);
            ui.label(egui::RichText::new(label).small().weak());
            ui.label(egui::RichText::new(format_bytes(bytes)).size(20.0).color(color).strong());
        });
}

fn render_requests_chart(ui: &mut egui::Ui, timeseries: &[crate::models::analytics::AnalyticsTimeseries]) {
    use egui_plot::{Line, Plot, PlotPoints};

    let cached_points: PlotPoints = timeseries
        .iter()
        .enumerate()
        .map(|(i, ts)| [i as f64, ts.requests.as_ref().and_then(|r| r.cached).unwrap_or(0) as f64])
        .collect();
    let uncached_points: PlotPoints = timeseries
        .iter()
        .enumerate()
        .map(|(i, ts)| [i as f64, ts.requests.as_ref().and_then(|r| r.uncached).unwrap_or(0) as f64])
        .collect();

    Plot::new("requests_chart")
        .height(180.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(cached_points).name("Cached").color(theme::SUCCESS));
            plot_ui.line(Line::new(uncached_points).name("Uncached").color(theme::WARNING));
        });
}

fn render_bandwidth_chart(ui: &mut egui::Ui, timeseries: &[crate::models::analytics::AnalyticsTimeseries]) {
    use egui_plot::{Line, Plot, PlotPoints};

    let bw_points: PlotPoints = timeseries
        .iter()
        .enumerate()
        .map(|(i, ts)| [i as f64, ts.bandwidth.as_ref().and_then(|b| b.all).unwrap_or(0) as f64])
        .collect();

    Plot::new("bandwidth_chart")
        .height(180.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(bw_points).name("Bandwidth").color(theme::INFO));
        });
}

pub fn load_analytics(state: &mut AppState, ctx: &egui::Context, zone_id: &str) {
    let client = match &state.client { Some(c) => c.clone(), None => return };
    let zid = zone_id.to_string();
    let period = state.analytics_period.clone();
    state.set_loading("Loading analytics...");
    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let params = match period.as_str() {
            "7d" => AnalyticsParams::last_7d(),
            _ => AnalyticsParams::last_24h(),
        };
        let result = client.get_analytics(&zid, &params).await;
        AsyncResult::AnalyticsLoaded(result)
    });
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { n.to_string() }
}

fn format_bytes(b: u64) -> String {
    if b >= 1_073_741_824 { format!("{:.1} GB", b as f64 / 1_073_741_824.0) }
    else if b >= 1_048_576 { format!("{:.1} MB", b as f64 / 1_048_576.0) }
    else if b >= 1_024 { format!("{:.1} KB", b as f64 / 1_024.0) }
    else { format!("{} B", b) }
}
