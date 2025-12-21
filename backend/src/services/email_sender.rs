use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::header::ContentType,
};
use serde::{Deserialize, Serialize};
use handlebars::Handlebars;
use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use crate::services::encryption::EncryptionService;

#[derive(Debug, Clone)]
pub struct EmailSender {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailJobPayload {
    pub campaign_lead_id: Uuid,
    pub campaign_id: Uuid,
    pub lead_id: Uuid,
    pub inbox_id: Uuid,
    pub email: String,
}

pub struct CampaignEmailSender {
    pool: Arc<PgPool>,
}

#[derive(Debug, sqlx::FromRow)]
struct InboxCredentials {
    id: Uuid,
    email: String,
    smtp_host: String,
    smtp_port: i32,
    smtp_username: String,
    smtp_password: Option<String>,
    smtp_password_encrypted: Option<Vec<u8>>,
    encryption_key_id: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct CampaignDetails {
    id: Uuid,
    name: String,
    workspace_id: Option<Uuid>,
}

#[derive(Debug, sqlx::FromRow)]
struct LeadDetails {
    id: Uuid,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    company: Option<String>,
    title: Option<String>,
}

impl CampaignEmailSender {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn send_campaign_email(&self, payload: &SendEmailJobPayload) -> Result<String, String> {
        // Get inbox credentials
        let inbox = sqlx::query_as::<_, InboxCredentials>(
            r#"
            SELECT id, email, smtp_host, smtp_port, smtp_username, smtp_password, 
                   smtp_password_encrypted, encryption_key_id
            FROM email_accounts WHERE id = $1
            "#
        )
        .bind(payload.inbox_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("Inbox not found")?;

        // Get campaign details
        let campaign = sqlx::query_as::<_, CampaignDetails>(
            "SELECT id, name, workspace_id FROM campaigns WHERE id = $1"
        )
        .bind(payload.campaign_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("Campaign not found")?;

        // Get lead details
        let lead = sqlx::query_as::<_, LeadDetails>(
            "SELECT id, email, first_name, last_name, company, title FROM leads WHERE id = $1"
        )
        .bind(payload.lead_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("Lead not found")?;

        // Decrypt SMTP password
        let smtp_password = self.get_smtp_password(&inbox)?;

        // Build personalized email
        let subject = self.personalize_text(
            &format!("Quick question about {}", lead.company.as_deref().unwrap_or("your company")),
            &lead
        );
        
        let body_html = self.build_email_body(&lead, &campaign);

        // Generate unsubscribe token and URL
        let unsubscribe_token = self.generate_unsubscribe_token(&lead, campaign.workspace_id);
        let unsubscribe_url = format!(
            "{}/unsubscribe?token={}",
            std::env::var("APP_URL").unwrap_or_else(|_| "https://app.outreachiq.com".to_string()),
            unsubscribe_token
        );

        // Build email with compliance headers
        let from = format!("{} <{}>", inbox.email.split('@').next().unwrap_or("Team"), inbox.email);
        let to_name = format!(
            "{} {}",
            lead.first_name.as_deref().unwrap_or(""),
            lead.last_name.as_deref().unwrap_or("")
        ).trim().to_string();
        let to = if to_name.is_empty() {
            lead.email.clone()
        } else {
            format!("{} <{}>", to_name, lead.email)
        };

        let email = Message::builder()
            .from(from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
            .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?)
            .subject(&subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(strip_html(&body_html)),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(body_html),
                    ),
            )
            .map_err(|e| format!("Failed to build email: {}", e))?;

        // Send via SMTP
        let creds = Credentials::new(inbox.smtp_username.clone(), smtp_password);

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&inbox.smtp_host)
                .map_err(|e| format!("Failed to create transport: {}", e))?
                .credentials(creds)
                .port(inbox.smtp_port as u16)
                .build();

        let response = mailer.send(email).await
            .map_err(|e| format!("SMTP error: {}", e))?;

        let message_id = response.message().collect::<Vec<_>>().join("");

        // Update campaign_leads status
        sqlx::query(
            "UPDATE campaign_leads SET status = 'sent', sent_at = NOW() WHERE id = $1"
        )
        .bind(payload.campaign_lead_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update campaign_lead: {}", e))?;

        // Update inbox sent_today counter
        sqlx::query(
            "UPDATE email_accounts SET sent_today = sent_today + 1 WHERE id = $1"
        )
        .bind(payload.inbox_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update inbox counter: {}", e))?;

        // Update campaign sent counter
        sqlx::query(
            "UPDATE campaigns SET sent = sent + 1 WHERE id = $1"
        )
        .bind(payload.campaign_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update campaign counter: {}", e))?;

        Ok(message_id)
    }

    fn get_smtp_password(&self, inbox: &InboxCredentials) -> Result<String, String> {
        // Try encrypted password first
        if let Some(encrypted) = &inbox.smtp_password_encrypted {
            if let Ok(enc_service) = EncryptionService::new() {
                if let Ok(decrypted) = enc_service.decrypt(encrypted) {
                    return Ok(decrypted);
                }
            }
        }
        
        // Fall back to plaintext (legacy)
        inbox.smtp_password.clone().ok_or_else(|| "No SMTP password available".to_string())
    }

    fn personalize_text(&self, text: &str, lead: &LeadDetails) -> String {
        text.replace("{{firstName}}", lead.first_name.as_deref().unwrap_or("there"))
            .replace("{{lastName}}", lead.last_name.as_deref().unwrap_or(""))
            .replace("{{company}}", lead.company.as_deref().unwrap_or("your company"))
            .replace("{{title}}", lead.title.as_deref().unwrap_or(""))
            .replace("{{email}}", &lead.email)
    }

    fn build_email_body(&self, lead: &LeadDetails, _campaign: &CampaignDetails) -> String {
        let first_name = lead.first_name.as_deref().unwrap_or("there");
        let company = lead.company.as_deref().unwrap_or("your company");
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <p>Hi {},</p>
        
        <p>I noticed {} and wanted to reach out to see if you'd be interested in a quick conversation.</p>
        
        <p>Would you be open to a brief 15-minute call this week?</p>
        
        <p>Best regards</p>
    </div>
</body>
</html>
"#, first_name, company)
    }

    fn generate_unsubscribe_token(&self, lead: &LeadDetails, workspace_id: Option<Uuid>) -> String {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        let data = format!("{}:{}:{}", 
            lead.id, 
            lead.email, 
            workspace_id.map(|w| w.to_string()).unwrap_or_default()
        );
        URL_SAFE_NO_PAD.encode(data.as_bytes())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl EmailSender {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_email: String,
        from_name: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        }
    }

    pub fn from_env() -> Option<Self> {
        Some(Self {
            smtp_host: std::env::var("SMTP_HOST").ok()?,
            smtp_port: std::env::var("SMTP_PORT").ok()?.parse().ok()?,
            smtp_username: std::env::var("SMTP_USERNAME").ok()?,
            smtp_password: std::env::var("SMTP_PASSWORD").ok()?,
            from_email: std::env::var("FROM_EMAIL").ok()?,
            from_name: std::env::var("FROM_NAME").unwrap_or_else(|_| "OutreachIQ".to_string()),
        })
    }

    pub async fn send(&self, request: SendEmailRequest) -> SendResult {
        let from = format!("{} <{}>", self.from_name, self.from_email);
        let to = match &request.to_name {
            Some(name) => format!("{} <{}>", name, request.to_email),
            None => request.to_email.clone(),
        };

        let email = match Message::builder()
            .from(from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(&request.subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(request.body_text.unwrap_or_else(|| strip_html(&request.body_html))),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(request.body_html),
                    ),
            ) {
            Ok(email) => email,
            Err(e) => {
                return SendResult {
                    success: false,
                    message_id: None,
                    error: Some(format!("Failed to build email: {}", e)),
                };
            }
        };

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            match AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host) {
                Ok(transport) => transport.credentials(creds).port(self.smtp_port).build(),
                Err(e) => {
                    return SendResult {
                        success: false,
                        message_id: None,
                        error: Some(format!("Failed to create transport: {}", e)),
                    };
                }
            };

        match mailer.send(email).await {
            Ok(response) => SendResult {
                success: true,
                message_id: Some(response.message().collect::<Vec<_>>().join("")),
                error: None,
            },
            Err(e) => SendResult {
                success: false,
                message_id: None,
                error: Some(format!("Failed to send email: {}", e)),
            },
        }
    }

    pub fn render_template(
        &self,
        template: &EmailTemplate,
        variables: &HashMap<String, String>,
    ) -> Result<(String, String, String), String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let subject = handlebars
            .render_template(&template.subject, variables)
            .map_err(|e| e.to_string())?;

        let body_html = handlebars
            .render_template(&template.body_html, variables)
            .map_err(|e| e.to_string())?;

        let body_text = handlebars
            .render_template(&template.body_text, variables)
            .map_err(|e| e.to_string())?;

        Ok((subject, body_html, body_text))
    }
}

fn strip_html(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").to_string()
}

#[derive(Debug, Clone)]
pub struct EmailTemplates;

impl EmailTemplates {
    pub fn cold_outreach() -> EmailTemplate {
        EmailTemplate {
            subject: "Quick question about {{company}}".to_string(),
            body_html: r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .signature { margin-top: 30px; color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <p>Hi {{firstName}},</p>
        
        <p>I noticed {{company}} is {{signal}} and wanted to reach out.</p>
        
        <p>{{customMessage}}</p>
        
        <p>Would you be open to a quick 15-minute call this week?</p>
        
        <div class="signature">
            <p>Best,<br>{{senderName}}<br>{{senderTitle}}</p>
        </div>
    </div>
</body>
</html>
"#.to_string(),
            body_text: r#"Hi {{firstName}},

I noticed {{company}} is {{signal}} and wanted to reach out.

{{customMessage}}

Would you be open to a quick 15-minute call this week?

Best,
{{senderName}}
{{senderTitle}}"#.to_string(),
        }
    }

    pub fn follow_up() -> EmailTemplate {
        EmailTemplate {
            subject: "Re: Quick question about {{company}}".to_string(),
            body_html: r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <p>Hi {{firstName}},</p>
        
        <p>Just wanted to follow up on my previous email. I understand you're busy, but I think this could be valuable for {{company}}.</p>
        
        <p>{{followUpMessage}}</p>
        
        <p>Let me know if you'd like to chat.</p>
        
        <p>Best,<br>{{senderName}}</p>
    </div>
</body>
</html>
"#.to_string(),
            body_text: r#"Hi {{firstName}},

Just wanted to follow up on my previous email. I understand you're busy, but I think this could be valuable for {{company}}.

{{followUpMessage}}

Let me know if you'd like to chat.

Best,
{{senderName}}"#.to_string(),
        }
    }

    pub fn warmup_email() -> EmailTemplate {
        EmailTemplate {
            subject: "{{subject}}".to_string(),
            body_html: r#"
<!DOCTYPE html>
<html>
<body>
    <p>{{body}}</p>
</body>
</html>
"#.to_string(),
            body_text: "{{body}}".to_string(),
        }
    }
}
