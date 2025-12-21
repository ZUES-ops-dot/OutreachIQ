use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SuppressionEntry {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub email: String,
    pub reason: String,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuppressionReason {
    Unsubscribed,
    Bounced,
    Complained,
    Manual,
}

impl SuppressionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SuppressionReason::Unsubscribed => "unsubscribed",
            SuppressionReason::Bounced => "bounced",
            SuppressionReason::Complained => "complained",
            SuppressionReason::Manual => "manual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unsubscribed" => Some(SuppressionReason::Unsubscribed),
            "bounced" => Some(SuppressionReason::Bounced),
            "complained" => Some(SuppressionReason::Complained),
            "manual" => Some(SuppressionReason::Manual),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddSuppressionRequest {
    pub email: String,
    pub reason: String,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UsageMetric {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub metric_type: String,
    pub count: i32,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    LeadsGenerated,
    EmailsSent,
    Verifications,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::LeadsGenerated => "leads_generated",
            MetricType::EmailsSent => "emails_sent",
            MetricType::Verifications => "verifications",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RateLimit {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub endpoint: String,
    pub requests_count: Option<i32>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub workspace_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub leads_generated: i32,
    pub emails_sent: i32,
    pub verifications: i32,
    pub lead_limit: i32,
    pub email_limit: i32,
    pub leads_remaining: i32,
    pub emails_remaining: i32,
}
