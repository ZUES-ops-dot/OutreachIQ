use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub plan_tier: Option<String>,
    pub monthly_lead_limit: Option<i32>,
    pub monthly_email_limit: Option<i32>,
    pub settings: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanTier {
    Starter,
    Professional,
    Business,
}

impl PlanTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanTier::Starter => "starter",
            PlanTier::Professional => "professional",
            PlanTier::Business => "business",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "starter" => Some(PlanTier::Starter),
            "professional" => Some(PlanTier::Professional),
            "business" => Some(PlanTier::Business),
            _ => None,
        }
    }

    pub fn lead_limit(&self) -> i32 {
        match self {
            PlanTier::Starter => 1000,
            PlanTier::Professional => 10000,
            PlanTier::Business => 50000,
        }
    }

    pub fn email_limit(&self) -> i32 {
        match self {
            PlanTier::Starter => 500,
            PlanTier::Professional => 5000,
            PlanTier::Business => 25000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WorkspaceMember {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl WorkspaceRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkspaceRole::Owner => "owner",
            WorkspaceRole::Admin => "admin",
            WorkspaceRole::Member => "member",
            WorkspaceRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(WorkspaceRole::Owner),
            "admin" => Some(WorkspaceRole::Admin),
            "member" => Some(WorkspaceRole::Member),
            "viewer" => Some(WorkspaceRole::Viewer),
            _ => None,
        }
    }

    pub fn can_write(&self) -> bool {
        matches!(self, WorkspaceRole::Owner | WorkspaceRole::Admin | WorkspaceRole::Member)
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, WorkspaceRole::Owner | WorkspaceRole::Admin)
    }

    pub fn is_owner(&self) -> bool {
        matches!(self, WorkspaceRole::Owner)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub slug: String,
    pub plan_tier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub plan_tier: Option<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: String,
}
