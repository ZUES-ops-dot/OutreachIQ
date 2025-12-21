use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub vertical: String,
    pub status: String,
    pub total_leads: i32,
    pub sent: i32,
    pub opened: i32,
    pub clicked: i32,
    pub replied: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Completed,
}

impl CampaignStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CampaignStatus::Draft => "draft",
            CampaignStatus::Active => "active",
            CampaignStatus::Paused => "paused",
            CampaignStatus::Completed => "completed",
        }
    }
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub vertical: String,
    pub lead_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CampaignLead {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub lead_id: Uuid,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub opened_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub replied_at: Option<DateTime<Utc>>,
    pub bounce_reason: Option<String>,
    pub open_count: Option<i32>,
    pub click_count: Option<i32>,
    pub bounce_type: Option<String>,
    pub unsubscribed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EmailAccount {
    pub id: Uuid,
    pub email: String,
    pub provider: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub warmup_status: String,
    pub daily_limit: i32,
    pub sent_today: i32,
    pub health_score: f32,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
    pub smtp_password_encrypted: Option<Vec<u8>>,
    pub encryption_key_id: Option<String>,
}
