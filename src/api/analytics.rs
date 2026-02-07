use anyhow::{Context, Result};
use chrono::{Duration, Utc};

use crate::api::client::CfClient;
use crate::models::analytics::*;

impl CfClient {
    // ==================== 分析数据 (GraphQL API) ====================

    /// 执行 GraphQL 查询
    async fn graphql_query(&self, query: &str, variables: serde_json::Value) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        let resp = self.post_raw("https://api.cloudflare.com/client/v4/graphql", &body).await?;

        // 检查 GraphQL 错误
        if let Some(errors) = resp.get("errors") {
            if let Some(arr) = errors.as_array() {
                if !arr.is_empty() {
                    let error_msgs: Vec<String> = arr.iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .map(|s| s.to_string())
                        .collect();
                    anyhow::bail!("GraphQL 错误: {}", error_msgs.join("; "));
                }
            }
        }

        Ok(resp)
    }

    /// 获取域名分析数据 (GraphQL)
    pub async fn get_analytics(
        &self,
        zone_id: &str,
        _params: &AnalyticsParams,
    ) -> Result<AnalyticsDashboard> {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let week_ago = now - Duration::days(7);

        // 日期格式: YYYY-MM-DD (用于 httpRequests1dGroups)
        let date_since = week_ago.format("%Y-%m-%d").to_string();
        let date_until = now.format("%Y-%m-%d").to_string();

        // 时间戳格式: ISO8601 (用于 httpRequests1hGroups)
        let datetime_since = yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let datetime_until = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let query = r#"
            query GetZoneAnalytics($zoneTag: String!, $dateSince: Date!, $dateUntil: Date!, $datetimeSince: Time!, $datetimeUntil: Time!) {
                viewer {
                    zones(filter: { zoneTag: $zoneTag }) {
                        httpRequests1dGroups(
                            limit: 7
                            filter: { date_geq: $dateSince, date_leq: $dateUntil }
                        ) {
                            sum {
                                requests
                                cachedRequests
                                encryptedRequests
                                bytes
                                cachedBytes
                                encryptedBytes
                                threats
                                pageViews
                            }
                            uniq {
                                uniques
                            }
                        }
                        httpRequests1hGroups(
                            limit: 24
                            filter: { datetime_geq: $datetimeSince, datetime_leq: $datetimeUntil }
                            orderBy: [datetime_ASC]
                        ) {
                            dimensions {
                                datetime
                            }
                            sum {
                                requests
                                cachedRequests
                                bytes
                                cachedBytes
                                threats
                            }
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "zoneTag": zone_id,
            "dateSince": date_since,
            "dateUntil": date_until,
            "datetimeSince": datetime_since,
            "datetimeUntil": datetime_until
        });

        let resp = self.graphql_query(query, variables).await?;

        // 解析响应
        let zones = resp
            .get("data")
            .and_then(|d| d.get("viewer"))
            .and_then(|v| v.get("zones"))
            .and_then(|z| z.as_array())
            .context("无法解析 GraphQL 响应")?;

        if zones.is_empty() {
            anyhow::bail!("未找到域名分析数据");
        }

        let zone = &zones[0];

        // 解析汇总数据
        let totals = self.parse_totals(zone);

        // 解析时间序列数据
        let timeseries = self.parse_timeseries(zone);

        Ok(AnalyticsDashboard { totals, timeseries })
    }

    /// 解析汇总数据
    fn parse_totals(&self, zone: &serde_json::Value) -> Option<AnalyticsTotals> {
        let groups = zone.get("httpRequests1dGroups")?.as_array()?;
        if groups.is_empty() {
            return None;
        }

        let group = &groups[0];
        let sum = group.get("sum")?;
        let uniq = group.get("uniq");

        let requests = Some(AnalyticsRequests {
            all: sum.get("requests").and_then(|v| v.as_u64()),
            cached: sum.get("cachedRequests").and_then(|v| v.as_u64()),
            uncached: {
                let all = sum.get("requests").and_then(|v| v.as_u64()).unwrap_or(0);
                let cached = sum.get("cachedRequests").and_then(|v| v.as_u64()).unwrap_or(0);
                Some(all.saturating_sub(cached))
            },
            ssl: Some(AnalyticsSslRequests {
                encrypted: sum.get("encryptedRequests").and_then(|v| v.as_u64()),
                unencrypted: {
                    let all = sum.get("requests").and_then(|v| v.as_u64()).unwrap_or(0);
                    let encrypted = sum.get("encryptedRequests").and_then(|v| v.as_u64()).unwrap_or(0);
                    Some(all.saturating_sub(encrypted))
                },
            }),
            http_status: None,
            content_type: None,
            country: None,
        });

        let bandwidth = Some(AnalyticsBandwidth {
            all: sum.get("bytes").and_then(|v| v.as_u64()),
            cached: sum.get("cachedBytes").and_then(|v| v.as_u64()),
            uncached: {
                let all = sum.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                let cached = sum.get("cachedBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                Some(all.saturating_sub(cached))
            },
            ssl: Some(AnalyticsSslBandwidth {
                encrypted: sum.get("encryptedBytes").and_then(|v| v.as_u64()),
                unencrypted: {
                    let all = sum.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let encrypted = sum.get("encryptedBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    Some(all.saturating_sub(encrypted))
                },
            }),
            content_type: None,
            country: None,
        });

        let threats = Some(AnalyticsThreats {
            all: sum.get("threats").and_then(|v| v.as_u64()),
            country: None,
            threat_type: None,
        });

        let pageviews = Some(AnalyticsPageviews {
            all: sum.get("pageViews").and_then(|v| v.as_u64()),
            search_engines: None,
        });

        let uniques = uniq.and_then(|u| {
            Some(AnalyticsUniques {
                all: u.get("uniques").and_then(|v| v.as_u64()),
            })
        });

        Some(AnalyticsTotals {
            requests,
            bandwidth,
            threats,
            pageviews,
            uniques,
        })
    }

    /// 解析时间序列数据
    fn parse_timeseries(&self, zone: &serde_json::Value) -> Option<Vec<AnalyticsTimeseries>> {
        let groups = zone.get("httpRequests1hGroups")?.as_array()?;

        let series: Vec<AnalyticsTimeseries> = groups.iter().filter_map(|group| {
            let dims = group.get("dimensions")?;
            let sum = group.get("sum")?;

            let datetime = dims.get("datetime").and_then(|v| v.as_str()).map(|s| s.to_string());

            Some(AnalyticsTimeseries {
                since: datetime.clone(),
                until: datetime,
                requests: Some(AnalyticsRequests {
                    all: sum.get("requests").and_then(|v| v.as_u64()),
                    cached: sum.get("cachedRequests").and_then(|v| v.as_u64()),
                    uncached: {
                        let all = sum.get("requests").and_then(|v| v.as_u64()).unwrap_or(0);
                        let cached = sum.get("cachedRequests").and_then(|v| v.as_u64()).unwrap_or(0);
                        Some(all.saturating_sub(cached))
                    },
                    ssl: None,
                    http_status: None,
                    content_type: None,
                    country: None,
                }),
                bandwidth: Some(AnalyticsBandwidth {
                    all: sum.get("bytes").and_then(|v| v.as_u64()),
                    cached: sum.get("cachedBytes").and_then(|v| v.as_u64()),
                    uncached: {
                        let all = sum.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                        let cached = sum.get("cachedBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                        Some(all.saturating_sub(cached))
                    },
                    ssl: None,
                    content_type: None,
                    country: None,
                }),
                threats: Some(AnalyticsThreats {
                    all: sum.get("threats").and_then(|v| v.as_u64()),
                    country: None,
                    threat_type: None,
                }),
                pageviews: None,
                uniques: None,
            })
        }).collect();

        if series.is_empty() {
            None
        } else {
            Some(series)
        }
    }

    /// 获取最近 24 小时的分析数据
    pub async fn get_analytics_24h(&self, zone_id: &str) -> Result<AnalyticsDashboard> {
        let params = AnalyticsParams::last_24h();
        self.get_analytics(zone_id, &params).await
    }

    /// 获取 DNS 分析数据 (GraphQL)
    pub async fn get_dns_analytics(
        &self,
        zone_id: &str,
        params: &AnalyticsParams,
    ) -> Result<serde_json::Value> {
        let (since, until) = params.get_time_range();

        let query = r#"
            query GetDnsAnalytics($zoneTag: String!, $since: Time!, $until: Time!) {
                viewer {
                    zones(filter: { zoneTag: $zoneTag }) {
                        dnsAnalyticsAdaptiveGroups(
                            limit: 100
                            filter: { datetime_geq: $since, datetime_leq: $until }
                        ) {
                            count
                            dimensions {
                                queryName
                                queryType
                                responseCode
                            }
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "zoneTag": zone_id,
            "since": since,
            "until": until
        });

        let resp = self.graphql_query(query, variables).await?;

        resp.get("data")
            .cloned()
            .context("获取 DNS 分析数据失败")
    }
}
