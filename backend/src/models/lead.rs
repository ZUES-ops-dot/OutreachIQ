use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Lead {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub linkedin_url: Option<String>,
    pub verification_status: String,
    pub confidence_score: f32,
    pub signals: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Pending,
    Valid,
    Invalid,
    Risky,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationStatus::Pending => "pending",
            VerificationStatus::Valid => "valid",
            VerificationStatus::Invalid => "invalid",
            VerificationStatus::Risky => "risky",
        }
    }
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeadSignals {
    pub recent_hiring: bool,
    pub funding_event: Option<String>,
    pub tech_stack: Vec<String>,
    pub company_size: Option<String>,
    pub growth_indicators: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeadSearchQuery {
    pub vertical: String,
    pub role: Option<String>,
    pub company_size: Option<String>,
    pub signals: Option<Vec<String>>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LeadResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub linkedin_url: Option<String>,
    pub verification_status: String,
    pub confidence_score: f32,
    pub signals: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
}

impl From<Lead> for LeadResponse {
    fn from(lead: Lead) -> Self {
        Self {
            id: lead.id,
            email: lead.email,
            first_name: lead.first_name,
            last_name: lead.last_name,
            company: lead.company,
            title: lead.title,
            linkedin_url: lead.linkedin_url,
            verification_status: lead.verification_status,
            confidence_score: lead.confidence_score,
            signals: lead.signals,
            created_at: lead.created_at,
            workspace_id: lead.workspace_id,
        }
    }
}
