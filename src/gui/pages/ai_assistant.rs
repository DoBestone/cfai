use eframe::egui;

use crate::ai::analyzer::AiAnalyzer;
use crate::gui::async_bridge::spawn_async;
use crate::gui::state::*;
use crate::gui::theme;

pub fn render(state: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("AI Assistant");
    ui.add_space(8.0);

    // Mode selector
    ui.horizontal(|ui| {
        ui.label("Mode:");
        for (mode, label) in &[
            (AiMode::Ask, "Ask"),
            (AiMode::AnalyzeDns, "DNS Analysis"),
            (AiMode::AnalyzeSecurity, "Security"),
            (AiMode::AnalyzePerformance, "Performance"),
            (AiMode::Troubleshoot, "Troubleshoot"),
            (AiMode::AutoConfig, "Auto Config"),
        ] {
            if ui.selectable_label(state.ai_mode == *mode, *label).clicked() {
                state.ai_mode = mode.clone();
            }
        }
    });
    ui.add_space(4.0);

    // Chat messages
    let scroll_height = ui.available_height() - 60.0;
    egui::ScrollArea::vertical()
        .id_salt("ai_chat")
        .max_height(scroll_height.max(200.0))
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if state.ai_messages.is_empty() {
                ui.label(egui::RichText::new("Ask me anything about Cloudflare...").weak());
            }
            for msg in &state.ai_messages {
                let is_user = msg.role == "user";
                let bg = if is_user {
                    egui::Color32::from_rgb(55, 65, 81)
                } else {
                    egui::Color32::from_rgb(31, 41, 55)
                };
                let align = if is_user {
                    egui::Layout::right_to_left(egui::Align::TOP)
                } else {
                    egui::Layout::left_to_right(egui::Align::TOP)
                };

                ui.with_layout(align, |ui| {
                    egui::Frame::none()
                        .fill(bg)
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_max_width(ui.available_width() * 0.8);
                            let prefix = if is_user { "You" } else { "AI" };
                            ui.label(egui::RichText::new(prefix).small().strong().color(
                                if is_user { theme::ACCENT } else { theme::SUCCESS },
                            ));
                            ui.label(&msg.content);

                            // Show suggested actions
                            if let Some(actions) = &msg.actions {
                                if !actions.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("Suggested Actions:").strong());
                                    for action in actions {
                                        let risk_color = match action.risk.as_str() {
                                            "low" => theme::SUCCESS,
                                            "medium" => theme::WARNING,
                                            "high" => theme::DANGER,
                                            _ => theme::INFO,
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("[{}]", action.risk)).color(risk_color).small());
                                            ui.label(egui::RichText::new(&action.description).small());
                                        });
                                    }
                                }
                            }
                        });
                });
                ui.add_space(4.0);
            }
        });

    // Input area
    ui.separator();
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.ai_input)
                .desired_width(ui.available_width() - 80.0)
                .hint_text("Type your question..."),
        );
        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (ui.button("Send").clicked() || enter_pressed) && !state.ai_input.trim().is_empty() {
            send_ai_message(state, ctx);
        }
    });
}

fn send_ai_message(state: &mut AppState, ctx: &egui::Context) {
    let input = state.ai_input.trim().to_string();
    if input.is_empty() { return; }

    state.ai_messages.push(AiChatMessage {
        role: "user".to_string(),
        content: input.clone(),
        actions: None,
    });
    state.ai_input.clear();

    let config = state.config.clone();
    let mode = state.ai_mode.clone();
    state.set_loading("AI thinking...");

    spawn_async(&state.tokio_handle, &state.tx, ctx, move || async move {
        let analyzer = match AiAnalyzer::new(&config) {
            Ok(a) => a,
            Err(e) => return AsyncResult::AiResponse(Err(e)),
        };
        let result = match mode {
            AiMode::Ask => analyzer.ask(&input).await,
            AiMode::AnalyzeDns => analyzer.analyze_dns(&input).await,
            AiMode::AnalyzeSecurity => analyzer.analyze_security(&input).await,
            AiMode::AnalyzePerformance => analyzer.analyze_performance(&input).await,
            AiMode::Troubleshoot => analyzer.troubleshoot(&input).await,
            AiMode::AutoConfig => analyzer.auto_config(&input).await,
        };
        AsyncResult::AiResponse(result)
    });
}
