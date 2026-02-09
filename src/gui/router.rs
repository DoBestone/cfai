use eframe::egui;

use super::state::{AppState, Page};
use super::theme::ACCENT;

pub fn render_sidebar(state: &mut AppState, ctx: &egui::Context) -> bool {
    let mut page_changed = false;

    egui::SidePanel::left("sidebar")
        .resizable(false)
        .default_width(180.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("CFAI").color(ACCENT).strong());
            });
            ui.label(egui::RichText::new("Cloudflare Manager").small().weak());
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            let nav_items: &[(Page, &str, &str)] = &[
                (Page::Dashboard, "\u{1F4CA}", "Dashboard"),
                (Page::Zone, "\u{1F310}", "Zones"),
                (Page::Dns, "\u{1F4E1}", "DNS"),
                (Page::Ssl, "\u{1F512}", "SSL/TLS"),
                (Page::Firewall, "\u{1F6E1}\u{FE0F}", "Firewall"),
                (Page::Cache, "\u{26A1}", "Cache"),
                (Page::PageRules, "\u{1F4C4}", "Page Rules"),
                (Page::Workers, "\u{2699}\u{FE0F}", "Workers"),
                (Page::Analytics, "\u{1F4C8}", "Analytics"),
                (Page::AiAssistant, "\u{1F916}", "AI Assistant"),
                (Page::Config, "\u{1F527}", "Settings"),
            ];

            for (page, icon, label) in nav_items {
                let is_selected = state.current_page == *page;
                let text = format!("{} {}", icon, label);
                let response = ui.selectable_label(is_selected, text);
                if response.clicked() && !is_selected {
                    state.current_page = page.clone();
                    page_changed = true;
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Zone selector
            ui.label(egui::RichText::new("Active Zone").small().strong());
            let selected_text = state
                .selected_zone
                .as_ref()
                .map(|z| z.name.as_str())
                .unwrap_or("Select zone...");

            egui::ComboBox::from_id_salt("zone_selector")
                .selected_text(selected_text)
                .width(160.0)
                .show_ui(ui, |ui| {
                    for zone in state.zones.clone() {
                        let is_sel = state
                            .selected_zone
                            .as_ref()
                            .map(|z| z.id == zone.id)
                            .unwrap_or(false);
                        if ui.selectable_label(is_sel, &zone.name).clicked() && !is_sel {
                            state.selected_zone = Some(zone);
                            page_changed = true;
                        }
                    }
                });

            // Connection status at bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                match state.connection_ok {
                    Some(true) => {
                        ui.label(
                            egui::RichText::new("\u{1F7E2} Connected")
                                .small()
                                .color(super::theme::SUCCESS),
                        );
                    }
                    Some(false) => {
                        ui.label(
                            egui::RichText::new("\u{1F534} Disconnected")
                                .small()
                                .color(super::theme::DANGER),
                        );
                    }
                    None => {
                        ui.label(egui::RichText::new("\u{1F7E1} Checking...").small());
                    }
                }
                if state.loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(&state.loading_label).small().weak(),
                        );
                    });
                }
            });
        });

    page_changed
}
