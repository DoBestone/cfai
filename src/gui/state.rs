use std::sync::mpsc;
use tokio::runtime::Handle;

use crate::api::client::CfClient;
use crate::config::settings::AppConfig;
use crate::models::analytics::AnalyticsDashboard;
use crate::models::dns::DnsRecord;
use crate::models::firewall::{FirewallRule, IpAccessRule, RateLimitRule};
use crate::models::page_rules::PageRule;
use crate::models::ssl::{SslCertificate, SslVerification};
use crate::models::workers::{KvNamespace, WorkerDomain, WorkerRoute, WorkerScript};
use crate::models::zone::{Zone, ZoneSetting};

use crate::ai::analyzer::{AnalysisResult, SuggestedAction};

/// Async result variants from background tasks
pub enum AsyncResult {
    ZonesLoaded(anyhow::Result<Vec<Zone>>),
    ZoneCreated(anyhow::Result<Zone>),
    ZoneDeleted(anyhow::Result<String>),
    ZoneToggled(anyhow::Result<Zone>),
    ZoneSettingsLoaded(anyhow::Result<Vec<ZoneSetting>>),

    DnsRecordsLoaded(anyhow::Result<Vec<DnsRecord>>),
    DnsRecordCreated(anyhow::Result<DnsRecord>),
    DnsRecordUpdated(anyhow::Result<DnsRecord>),
    DnsRecordDeleted(anyhow::Result<String>),
    DnsExported(anyhow::Result<String>),

    SslStatusLoaded(anyhow::Result<(String, bool, String)>),
    SslModeSet(anyhow::Result<String>),
    SslCertificatesLoaded(anyhow::Result<Vec<SslCertificate>>),
    SslVerificationsLoaded(anyhow::Result<Vec<SslVerification>>),
    SslToggled(anyhow::Result<String>),

    FirewallRulesLoaded(anyhow::Result<Vec<FirewallRule>>),
    IpAccessRulesLoaded(anyhow::Result<Vec<IpAccessRule>>),
    IpRuleCreated(anyhow::Result<String>),
    IpRuleDeleted(anyhow::Result<String>),
    SecurityLevelLoaded(anyhow::Result<String>),
    RateLimitsLoaded(anyhow::Result<Vec<RateLimitRule>>),
    FirewallActionDone(anyhow::Result<String>),

    CacheStatusLoaded(anyhow::Result<(String, u32, bool)>),
    CachePurged(anyhow::Result<String>),
    CacheActionDone(anyhow::Result<String>),

    PageRulesLoaded(anyhow::Result<Vec<PageRule>>),
    PageRuleCreated(anyhow::Result<String>),
    PageRuleDeleted(anyhow::Result<String>),

    WorkersLoaded(anyhow::Result<Vec<WorkerScript>>),
    WorkerRoutesLoaded(anyhow::Result<Vec<WorkerRoute>>),
    KvNamespacesLoaded(anyhow::Result<Vec<KvNamespace>>),
    WorkerDomainsLoaded(anyhow::Result<Vec<WorkerDomain>>),
    WorkerDeleted(anyhow::Result<String>),

    AnalyticsLoaded(anyhow::Result<AnalyticsDashboard>),

    AiResponse(anyhow::Result<AnalysisResult>),

    ConfigSaved(anyhow::Result<()>),
    TokenVerified(anyhow::Result<bool>),
}
/// Navigation pages
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Dashboard,
    Zone,
    Dns,
    Ssl,
    Firewall,
    Cache,
    PageRules,
    Workers,
    Analytics,
    AiAssistant,
    Config,
}

/// Notification level
#[derive(Debug, Clone, PartialEq)]
pub enum NotifLevel {
    Success,
    Error,
    Warning,
    Info,
}

/// Toast notification
pub struct Notification {
    pub message: String,
    pub level: NotifLevel,
    pub created_at: std::time::Instant,
}

impl Notification {
    pub fn new(message: String, level: NotifLevel) -> Self {
        Self { message, level, created_at: std::time::Instant::now() }
    }
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > std::time::Duration::from_secs(5)
    }
}

/// AI chat message
#[derive(Clone)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
    pub actions: Option<Vec<SuggestedAction>>,
}

/// AI mode
#[derive(Debug, Clone, PartialEq)]
pub enum AiMode {
    Ask,
    AnalyzeDns,
    AnalyzeSecurity,
    AnalyzePerformance,
    Troubleshoot,
    AutoConfig,
}

/// DNS add form
pub struct DnsAddForm {
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: String,
    pub proxied: bool,
    pub priority: String,
    pub comment: String,
}

impl Default for DnsAddForm {
    fn default() -> Self {
        Self {
            record_type: "A".to_string(),
            name: String::new(),
            content: String::new(),
            ttl: "1".to_string(),
            proxied: true,
            priority: String::new(),
            comment: String::new(),
        }
    }
}

/// DNS edit form
pub struct DnsEditForm {
    pub record_id: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: String,
    pub proxied: bool,
    pub priority: String,
    pub comment: String,
}

/// Redirect form for page rules
pub struct RedirectForm {
    pub url_pattern: String,
    pub redirect_url: String,
    pub status_code: u16,
}

impl Default for RedirectForm {
    fn default() -> Self {
        Self {
            url_pattern: String::new(),
            redirect_url: String::new(),
            status_code: 301,
        }
    }
}

/// Workers tab
#[derive(Debug, Clone, PartialEq)]
pub enum WorkersTab {
    Scripts,
    Routes,
    Kv,
    Domains,
}

/// Confirm dialog
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
}

/// Actions that can be confirmed
#[derive(Clone)]
pub enum ConfirmAction {
    DeleteZone(String),
    DeleteDnsRecord(String, String),
    DeletePageRule(String, String),
    DeleteWorker(String),
    PurgeAllCache(String),
    DeleteIpRule(String, String),
}

/// Full application state
pub struct AppState {
    // Infrastructure
    pub config: AppConfig,
    pub client: Option<CfClient>,
    pub tokio_handle: Handle,
    pub tx: mpsc::Sender<AsyncResult>,
    pub rx: mpsc::Receiver<AsyncResult>,
    pub loading: bool,
    pub loading_label: String,
    pub notifications: Vec<Notification>,
    pub connection_ok: Option<bool>,

    // Navigation
    pub current_page: Page,
    pub zones: Vec<Zone>,
    pub selected_zone: Option<Zone>,
    pub zones_loaded: bool,

    // Zone page
    pub zone_search: String,
    pub zone_add_domain: String,
    pub zone_settings: Vec<ZoneSetting>,

    // DNS page
    pub dns_records: Vec<DnsRecord>,
    pub dns_filter_type: String,
    pub dns_search: String,
    pub dns_add_form: DnsAddForm,
    pub dns_edit_form: Option<DnsEditForm>,
    pub dns_show_add: bool,

    // SSL page
    pub ssl_mode: String,
    pub ssl_always_https: bool,
    pub ssl_min_tls: String,
    pub ssl_certificates: Vec<SslCertificate>,
    pub ssl_verifications: Vec<SslVerification>,

    // Firewall page
    pub firewall_rules: Vec<FirewallRule>,
    pub ip_access_rules: Vec<IpAccessRule>,
    pub security_level: String,
    pub rate_limits: Vec<RateLimitRule>,
    pub fw_ip_input: String,
    pub fw_note_input: String,

    // Cache page
    pub cache_level: String,
    pub browser_cache_ttl: u32,
    pub dev_mode_on: bool,
    pub purge_urls_input: String,

    // Page Rules page
    pub page_rules: Vec<PageRule>,
    pub redirect_form: RedirectForm,

    // Workers page
    pub worker_scripts: Vec<WorkerScript>,
    pub worker_routes: Vec<WorkerRoute>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub worker_domains: Vec<WorkerDomain>,
    pub workers_tab: WorkersTab,

    // Analytics page
    pub analytics: Option<AnalyticsDashboard>,
    pub analytics_period: String,

    // AI Assistant page
    pub ai_messages: Vec<AiChatMessage>,
    pub ai_input: String,
    pub ai_mode: AiMode,

    // Config page
    pub config_edit: AppConfig,
    pub config_show_secrets: bool,

    // Confirm dialog
    pub confirm_dialog: Option<ConfirmDialog>,
}

impl AppState {
    pub fn new(config: AppConfig, client: Option<CfClient>, handle: Handle) -> Self {
        let (tx, rx) = mpsc::channel();
        let config_edit = config.clone();
        Self {
            config,
            client,
            tokio_handle: handle,
            tx,
            rx,
            loading: false,
            loading_label: String::new(),
            notifications: Vec::new(),
            connection_ok: None,
            current_page: Page::Dashboard,
            zones: Vec::new(),
            selected_zone: None,
            zones_loaded: false,
            zone_search: String::new(),
            zone_add_domain: String::new(),
            zone_settings: Vec::new(),
            dns_records: Vec::new(),
            dns_filter_type: String::new(),
            dns_search: String::new(),
            dns_add_form: DnsAddForm::default(),
            dns_edit_form: None,
            dns_show_add: false,
            ssl_mode: String::new(),
            ssl_always_https: false,
            ssl_min_tls: "1.0".to_string(),
            ssl_certificates: Vec::new(),
            ssl_verifications: Vec::new(),
            firewall_rules: Vec::new(),
            ip_access_rules: Vec::new(),
            security_level: String::new(),
            rate_limits: Vec::new(),
            fw_ip_input: String::new(),
            fw_note_input: String::new(),
            cache_level: String::new(),
            browser_cache_ttl: 0,
            dev_mode_on: false,
            purge_urls_input: String::new(),
            page_rules: Vec::new(),
            redirect_form: RedirectForm::default(),
            worker_scripts: Vec::new(),
            worker_routes: Vec::new(),
            kv_namespaces: Vec::new(),
            worker_domains: Vec::new(),
            workers_tab: WorkersTab::Scripts,
            analytics: None,
            analytics_period: "24h".to_string(),
            ai_messages: Vec::new(),
            ai_input: String::new(),
            ai_mode: AiMode::Ask,
            config_edit,
            config_show_secrets: false,
            confirm_dialog: None,
        }
    }

    pub fn notify(&mut self, msg: impl Into<String>, level: NotifLevel) {
        self.notifications.push(Notification::new(msg.into(), level));
    }

    pub fn set_loading(&mut self, label: &str) {
        self.loading = true;
        self.loading_label = label.to_string();
    }

    pub fn clear_loading(&mut self) {
        self.loading = false;
        self.loading_label.clear();
    }

    pub fn zone_id(&self) -> Option<String> {
        self.selected_zone.as_ref().map(|z| z.id.clone())
    }
}
