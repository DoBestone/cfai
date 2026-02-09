mod async_bridge;
mod pages;
mod router;
mod state;
mod theme;
mod widgets;

use anyhow::Result;
use eframe::egui;

use crate::api::client::{AuthMethod, CfClient};
use crate::config::settings::AppConfig;

use state::*;

/// Main GUI application
struct CfaiApp {
    state: AppState,
}

impl eframe::App for CfaiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drain async results
        self.drain_results();

        // 2. Expire notifications
        self.state.notifications.retain(|n| !n.is_expired());

        // 3. Render sidebar
        let page_changed = router::render_sidebar(&mut self.state, ctx);

        // 4. Status bar
        widgets::status_bar::render_status_bar(&self.state, ctx);

        // 5. Central panel with current page
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.state.current_page {
                    Page::Dashboard => pages::dashboard::render(&mut self.state, ctx, ui),
                    Page::Zone => pages::zone::render(&mut self.state, ctx, ui),
                    Page::Dns => pages::dns::render(&mut self.state, ctx, ui),
                    Page::Ssl => pages::ssl::render(&mut self.state, ctx, ui),
                    Page::Firewall => pages::firewall::render(&mut self.state, ctx, ui),
                    Page::Cache => pages::cache::render(&mut self.state, ctx, ui),
                    Page::PageRules => pages::page_rules::render(&mut self.state, ctx, ui),
                    Page::Workers => pages::workers::render(&mut self.state, ctx, ui),
                    Page::Analytics => pages::analytics::render(&mut self.state, ctx, ui),
                    Page::AiAssistant => pages::ai_assistant::render(&mut self.state, ctx, ui),
                    Page::Config => pages::config::render(&mut self.state, ctx, ui),
                }
            });
        });

        // 6. Overlays
        widgets::notification::render_notifications(&mut self.state, ctx);
        widgets::confirm_dialog::render_confirm_dialog(&mut self.state, ctx);

        // 7. Auto-load zones on first frame
        if !self.state.zones_loaded && self.state.client.is_some() {
            self.state.zones_loaded = true;
            pages::dashboard::load_zones(&mut self.state, ctx);
            // Verify connection
            let client = self.state.client.as_ref().unwrap().clone();
            async_bridge::spawn_async(
                &self.state.tokio_handle,
                &self.state.tx,
                ctx,
                move || async move {
                    let result = client.verify_token().await;
                    AsyncResult::TokenVerified(result)
                },
            );
        }

        // 8. Load data when page changes or zone changes
        if page_changed {
            self.on_page_enter(ctx);
        }
    }
}
impl CfaiApp {
    fn on_page_enter(&mut self, ctx: &egui::Context) {
        let zone_id = self.state.zone_id();
        match self.state.current_page {
            Page::Dashboard => {
                pages::dashboard::load_zones(&mut self.state, ctx);
            }
            Page::Zone => {
                // Zones already loaded from dashboard
            }
            Page::Dns => {
                if let Some(zid) = &zone_id {
                    pages::dns::load_dns(&mut self.state, ctx, zid);
                }
            }
            Page::Ssl => {
                if let Some(zid) = &zone_id {
                    pages::ssl::load_ssl_status(&mut self.state, ctx, zid);
                }
            }
            Page::Firewall => {
                if let Some(zid) = &zone_id {
                    pages::firewall::load_firewall(&mut self.state, ctx, zid);
                }
            }
            Page::Cache => {
                if let Some(zid) = &zone_id {
                    pages::cache::load_cache_status(&mut self.state, ctx, zid);
                }
            }
            Page::PageRules => {
                if let Some(zid) = &zone_id {
                    pages::page_rules::load_page_rules(&mut self.state, ctx, zid);
                }
            }
            Page::Workers => {
                if let Some(aid) = &self.state.config.cloudflare.account_id.clone() {
                    pages::workers::load_workers(&mut self.state, ctx, aid);
                }
            }
            Page::Analytics => {
                if let Some(zid) = &zone_id {
                    pages::analytics::load_analytics(&mut self.state, ctx, zid);
                }
            }
            Page::AiAssistant | Page::Config => {}
        }
    }

    fn drain_results(&mut self) {
        while let Ok(result) = self.state.rx.try_recv() {
            self.state.clear_loading();
            match result {
                AsyncResult::ZonesLoaded(res) => match res {
                    Ok(zones) => {
                        self.state.zones = zones;
                        if self.state.selected_zone.is_none() {
                            if let Some(first) = self.state.zones.first() {
                                self.state.selected_zone = Some(first.clone());
                            }
                        }
                    }
                    Err(e) => self.state.notify(format!("Load zones failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::ZoneCreated(res) => match res {
                    Ok(zone) => {
                        self.state.notify(format!("Zone '{}' created", zone.name), NotifLevel::Success);
                        self.state.zones.push(zone);
                    }
                    Err(e) => self.state.notify(format!("Create zone failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::ZoneDeleted(res) => match res {
                    Ok(id) => {
                        self.state.zones.retain(|z| z.id != id);
                        if self.state.selected_zone.as_ref().map(|z| z.id == id).unwrap_or(false) {
                            self.state.selected_zone = self.state.zones.first().cloned();
                        }
                        self.state.notify("Zone deleted", NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Delete zone failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::ZoneToggled(res) => match res {
                    Ok(zone) => {
                        let paused = zone.paused.unwrap_or(false);
                        self.state.notify(format!("Zone {} {}", zone.name, if paused { "paused" } else { "resumed" }), NotifLevel::Success);
                        if let Some(z) = self.state.zones.iter_mut().find(|z| z.id == zone.id) {
                            *z = zone;
                        }
                    }
                    Err(e) => self.state.notify(format!("Toggle failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::ZoneSettingsLoaded(res) => match res {
                    Ok(settings) => self.state.zone_settings = settings,
                    Err(e) => self.state.notify(format!("Load settings failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::DnsRecordsLoaded(res) => match res {
                    Ok(records) => self.state.dns_records = records,
                    Err(e) => self.state.notify(format!("Load DNS failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::DnsRecordCreated(res) => match res {
                    Ok(record) => {
                        self.state.notify(format!("DNS record '{}' created", record.name), NotifLevel::Success);
                        self.state.dns_records.push(record);
                        self.state.dns_show_add = false;
                    }
                    Err(e) => self.state.notify(format!("Create DNS failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::DnsRecordUpdated(res) => match res {
                    Ok(record) => {
                        self.state.notify(format!("DNS record '{}' updated", record.name), NotifLevel::Success);
                        if let Some(r) = self.state.dns_records.iter_mut().find(|r| r.id == record.id) {
                            *r = record;
                        }
                        self.state.dns_edit_form = None;
                    }
                    Err(e) => self.state.notify(format!("Update DNS failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::DnsRecordDeleted(res) => match res {
                    Ok(id) => {
                        self.state.dns_records.retain(|r| r.id.as_deref() != Some(&id));
                        self.state.notify("DNS record deleted", NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Delete DNS failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::DnsExported(res) => match res {
                    Ok(data) => {
                        if let Ok(mut clip) = arboard::Clipboard::new() {
                            let _ = clip.set_text(&data);
                            self.state.notify("DNS exported to clipboard", NotifLevel::Success);
                        } else {
                            self.state.notify("Export done but clipboard unavailable", NotifLevel::Warning);
                        }
                    }
                    Err(e) => self.state.notify(format!("Export failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SslStatusLoaded(res) => match res {
                    Ok((mode, https, min_tls)) => {
                        self.state.ssl_mode = mode;
                        self.state.ssl_always_https = https;
                        self.state.ssl_min_tls = min_tls;
                    }
                    Err(e) => self.state.notify(format!("Load SSL failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SslModeSet(res) => match res {
                    Ok(mode) => {
                        self.state.ssl_mode = mode.clone();
                        self.state.notify(format!("SSL mode set to {}", mode), NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Set SSL mode failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SslCertificatesLoaded(res) => match res {
                    Ok(certs) => self.state.ssl_certificates = certs,
                    Err(e) => self.state.notify(format!("Load certs failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SslVerificationsLoaded(res) => match res {
                    Ok(v) => self.state.ssl_verifications = v,
                    Err(e) => self.state.notify(format!("Load verifications failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SslToggled(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("SSL toggle failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::FirewallRulesLoaded(res) => match res {
                    Ok(rules) => self.state.firewall_rules = rules,
                    Err(e) => self.state.notify(format!("Load firewall failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::IpAccessRulesLoaded(res) => match res {
                    Ok(rules) => self.state.ip_access_rules = rules,
                    Err(e) => self.state.notify(format!("Load IP rules failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::IpRuleCreated(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("IP rule failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::IpRuleDeleted(res) => match res {
                    Ok(id) => {
                        self.state.ip_access_rules.retain(|r| r.id.as_deref() != Some(&id));
                        self.state.notify("IP rule deleted", NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Delete IP rule failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::SecurityLevelLoaded(res) => match res {
                    Ok(level) => self.state.security_level = level,
                    Err(e) => self.state.notify(format!("Load security level failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::RateLimitsLoaded(res) => match res {
                    Ok(limits) => self.state.rate_limits = limits,
                    Err(e) => self.state.notify(format!("Load rate limits failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::FirewallActionDone(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("Firewall action failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::CacheStatusLoaded(res) => match res {
                    Ok((level, ttl, dev)) => {
                        self.state.cache_level = level;
                        self.state.browser_cache_ttl = ttl;
                        self.state.dev_mode_on = dev;
                    }
                    Err(e) => self.state.notify(format!("Load cache failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::CachePurged(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("Purge failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::CacheActionDone(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("Cache action failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::PageRulesLoaded(res) => match res {
                    Ok(rules) => self.state.page_rules = rules,
                    Err(e) => self.state.notify(format!("Load page rules failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::PageRuleCreated(res) => match res {
                    Ok(msg) => self.state.notify(msg, NotifLevel::Success),
                    Err(e) => self.state.notify(format!("Create page rule failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::PageRuleDeleted(res) => match res {
                    Ok(id) => {
                        self.state.page_rules.retain(|r| r.id.as_deref() != Some(&id));
                        self.state.notify("Page rule deleted", NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Delete page rule failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::WorkersLoaded(res) => match res {
                    Ok(scripts) => self.state.worker_scripts = scripts,
                    Err(e) => self.state.notify(format!("Load workers failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::WorkerRoutesLoaded(res) => match res {
                    Ok(routes) => self.state.worker_routes = routes,
                    Err(e) => self.state.notify(format!("Load routes failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::KvNamespacesLoaded(res) => match res {
                    Ok(ns) => self.state.kv_namespaces = ns,
                    Err(e) => self.state.notify(format!("Load KV failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::WorkerDomainsLoaded(res) => match res {
                    Ok(domains) => self.state.worker_domains = domains,
                    Err(e) => self.state.notify(format!("Load worker domains failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::WorkerDeleted(res) => match res {
                    Ok(name) => {
                        self.state.worker_scripts.retain(|s| s.id.as_deref() != Some(&name));
                        self.state.notify(format!("Worker '{}' deleted", name), NotifLevel::Success);
                    }
                    Err(e) => self.state.notify(format!("Delete worker failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::AnalyticsLoaded(res) => match res {
                    Ok(dashboard) => self.state.analytics = Some(dashboard),
                    Err(e) => self.state.notify(format!("Load analytics failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::AiResponse(res) => match res {
                    Ok(result) => {
                        self.state.ai_messages.push(AiChatMessage {
                            role: "assistant".to_string(),
                            content: result.content,
                            actions: result.actions,
                        });
                    }
                    Err(e) => {
                        self.state.ai_messages.push(AiChatMessage {
                            role: "assistant".to_string(),
                            content: format!("Error: {}", e),
                            actions: None,
                        });
                    }
                },
                AsyncResult::ConfigSaved(res) => match res {
                    Ok(()) => self.state.notify("Config saved", NotifLevel::Success),
                    Err(e) => self.state.notify(format!("Save config failed: {}", e), NotifLevel::Error),
                },
                AsyncResult::TokenVerified(res) => match res {
                    Ok(valid) => {
                        self.state.connection_ok = Some(valid);
                        if valid {
                            self.state.notify("Token verified", NotifLevel::Success);
                        } else {
                            self.state.notify("Token invalid", NotifLevel::Error);
                        }
                    }
                    Err(e) => {
                        self.state.connection_ok = Some(false);
                        self.state.notify(format!("Verify failed: {}", e), NotifLevel::Error);
                    }
                },
            }
        }
    }
}

/// Launch the GUI window
pub fn launch_gui() -> Result<()> {
    let config = AppConfig::load()?.merge_env();

    let client = create_client_if_configured(&config);

    let handle = tokio::runtime::Handle::current();

    let state = AppState::new(config, client, handle);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("CFAI - Cloudflare Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "CFAI",
        options,
        Box::new(|cc| {
            theme::setup_theme(&cc.egui_ctx);
            Ok(Box::new(CfaiApp { state }))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

fn create_client_if_configured(config: &AppConfig) -> Option<CfClient> {
    if let Some(token) = &config.cloudflare.api_token {
        CfClient::new(AuthMethod::ApiToken(token.clone())).ok()
    } else if let (Some(email), Some(key)) = (&config.cloudflare.email, &config.cloudflare.api_key) {
        CfClient::new(AuthMethod::ApiKey {
            email: email.clone(),
            key: key.clone(),
        })
        .ok()
    } else {
        None
    }
}
