use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailAccountHealth {
    pub account_id: Uuid,
    pub email: String,
    pub health_score: f32,
    pub daily_limit: i32,
    pub sent_today: i32,
    pub bounce_rate: f32,
    pub spam_rate: f32,
    pub warmup_status: WarmupStatus,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarmupStatus {
    NotStarted,
    InProgress,
    Completed,
    Paused,
}

impl WarmupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WarmupStatus::NotStarted => "not_started",
            WarmupStatus::InProgress => "in_progress",
            WarmupStatus::Completed => "completed",
            WarmupStatus::Paused => "paused",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliverabilityReport {
    pub total_sent: i32,
    pub delivered: i32,
    pub bounced: i32,
    pub spam_complaints: i32,
    pub delivery_rate: f32,
    pub bounce_rate: f32,
    pub spam_rate: f32,
    pub recommendations: Vec<String>,
}

pub struct DeliverabilityService {
    warmup_schedule: Vec<i32>,
}

impl DeliverabilityService {
    pub fn new() -> Self {
        // Warmup schedule: emails per day for 30 days
        let warmup_schedule = vec![
            5, 7, 10, 12, 15, 18, 22, 26, 30, 35,
            40, 45, 50, 55, 60, 65, 70, 75, 80, 85,
            90, 95, 100, 110, 120, 130, 140, 150, 175, 200,
        ];
        Self { warmup_schedule }
    }

    /// Calculate the daily sending limit based on warmup day
    pub fn get_daily_limit(&self, warmup_day: usize) -> i32 {
        if warmup_day >= self.warmup_schedule.len() {
            return 200; // Max daily limit after warmup
        }
        self.warmup_schedule[warmup_day]
    }

    /// Check if an email account is healthy enough to send
    pub fn is_healthy(&self, health: &EmailAccountHealth) -> bool {
        health.health_score >= 70.0 
            && health.bounce_rate < 0.05 
            && health.spam_rate < 0.01
    }

    /// Calculate health score based on metrics
    pub fn calculate_health_score(
        &self,
        bounce_rate: f32,
        spam_rate: f32,
        reply_rate: f32,
    ) -> f32 {
        let mut score = 100.0;
        
        // Penalize for bounces
        score -= bounce_rate * 200.0; // 5% bounce = -10 points
        
        // Heavily penalize for spam
        score -= spam_rate * 500.0; // 1% spam = -5 points
        
        // Reward for replies
        score += reply_rate * 50.0; // 10% reply = +5 points
        
        score.max(0.0).min(100.0)
    }

    /// Generate deliverability report
    pub fn generate_report(
        &self,
        total_sent: i32,
        delivered: i32,
        bounced: i32,
        spam_complaints: i32,
    ) -> DeliverabilityReport {
        let delivery_rate = if total_sent > 0 {
            delivered as f32 / total_sent as f32
        } else {
            0.0
        };
        
        let bounce_rate = if total_sent > 0 {
            bounced as f32 / total_sent as f32
        } else {
            0.0
        };
        
        let spam_rate = if total_sent > 0 {
            spam_complaints as f32 / total_sent as f32
        } else {
            0.0
        };

        let mut recommendations = Vec::new();
        
        if bounce_rate > 0.03 {
            recommendations.push("High bounce rate detected. Verify email list quality.".to_string());
        }
        
        if spam_rate > 0.005 {
            recommendations.push("Spam complaints detected. Review email content and targeting.".to_string());
        }
        
        if delivery_rate < 0.95 {
            recommendations.push("Delivery rate below optimal. Check sender reputation.".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Deliverability metrics are healthy. Keep up the good work!".to_string());
        }

        DeliverabilityReport {
            total_sent,
            delivered,
            bounced,
            spam_complaints,
            delivery_rate,
            bounce_rate,
            spam_rate,
            recommendations,
        }
    }

    /// Get warmup recommendations
    pub fn get_warmup_recommendations(&self, day: usize, current_health: f32) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if day < 14 {
            recommendations.push("Continue warmup phase. Avoid sending cold emails yet.".to_string());
        }
        
        if current_health < 80.0 {
            recommendations.push("Health score is low. Reduce sending volume temporarily.".to_string());
        }
        
        recommendations.push(format!(
            "Recommended daily limit for day {}: {} emails",
            day + 1,
            self.get_daily_limit(day)
        ));
        
        recommendations
    }

    /// Check SPF, DKIM, DMARC records (mock implementation)
    pub async fn check_domain_authentication(&self, domain: &str) -> DomainAuthStatus {
        // In production: Actually check DNS records
        DomainAuthStatus {
            domain: domain.to_string(),
            spf_valid: true,
            dkim_valid: true,
            dmarc_valid: true,
            recommendations: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainAuthStatus {
    pub domain: String,
    pub spf_valid: bool,
    pub dkim_valid: bool,
    pub dmarc_valid: bool,
    pub recommendations: Vec<String>,
}

impl Default for DeliverabilityService {
    fn default() -> Self {
        Self::new()
    }
}
