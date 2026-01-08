use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

const CLASSIFICATION_PROMPT: &str = r#"Classify this cold email reply into ONE category:
- interested: Shows clear interest ("Tell me more", "Let's chat", "Send calendar", "Yes", "Sounds good", positive engagement)
- maybe_later: Timing issue but not negative ("Not now", "Check back Q2", "Timing bad", "Maybe next quarter")
- objection: Has questions or concerns ("How much?", "Who else uses this?", "What's the pricing?", "Need more info")
- negative: Wants to stop ("Unsubscribe", "Stop", "Remove me", angry tone, explicit rejection)
- auto_reply: Automated response (OOO, bounce-back, vacation, auto-responder)

Reply text:
{reply_text}

Return ONLY the category name (one word, lowercase). Nothing else."#;

pub async fn classify_reply(reply_text: &str) -> Result<(String, f32), String> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .or_else(|_| env::var("CLAUDE_API_KEY"))
        .map_err(|_| "ANTHROPIC_API_KEY or CLAUDE_API_KEY not set")?;

    let client = Client::new();
    
    let prompt = CLASSIFICATION_PROMPT.replace("{reply_text}", reply_text);
    
    let request = ClaudeRequest {
        model: "claude-3-haiku-20240307".to_string(),  // Fast and cheap for classification
        max_tokens: 20,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to call Claude API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Claude API error {}: {}", status, error_text));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    let classification = claude_response
        .content
        .first()
        .map(|c| c.text.trim().to_lowercase())
        .unwrap_or_else(|| "auto_reply".to_string());

    // Validate the classification
    let valid_intents = ["interested", "maybe_later", "objection", "negative", "auto_reply"];
    let intent = if valid_intents.contains(&classification.as_str()) {
        classification
    } else {
        // Try to extract a valid intent from the response
        valid_intents
            .iter()
            .find(|&&i| classification.contains(i))
            .map(|&s| s.to_string())
            .unwrap_or_else(|| "auto_reply".to_string())
    };

    // Confidence is high for Claude classifications
    let confidence = 0.85_f32;

    Ok((intent, confidence))
}

pub fn classify_reply_simple(reply_text: &str) -> (String, f32) {
    let text = reply_text.to_lowercase();
    
    // Auto-reply patterns
    let auto_reply_patterns = [
        "out of office", "ooo", "vacation", "away from", "auto-reply",
        "automatic reply", "i am currently out", "will return", "limited access",
        "this is an automated", "do not reply", "undeliverable", "delivery failed",
        "mailbox full", "no longer with"
    ];
    
    for pattern in auto_reply_patterns {
        if text.contains(pattern) {
            return ("auto_reply".to_string(), 0.95);
        }
    }

    // Negative patterns
    let negative_patterns = [
        "unsubscribe", "stop emailing", "remove me", "take me off", "not interested",
        "stop contacting", "do not contact", "leave me alone", "spam", "reported",
        "harassment", "legal action", "cease and desist", "f**k", "fuck off"
    ];
    
    for pattern in negative_patterns {
        if text.contains(pattern) {
            return ("negative".to_string(), 0.90);
        }
    }

    // Interested patterns
    let interested_patterns = [
        "let's chat", "let's talk", "schedule a call", "book a meeting",
        "send me your calendar", "interested", "tell me more", "sounds good",
        "yes", "love to learn more", "set up a time", "when are you free",
        "happy to connect", "let's do it"
    ];
    
    for pattern in interested_patterns {
        if text.contains(pattern) {
            return ("interested".to_string(), 0.85);
        }
    }

    // Maybe later patterns
    let maybe_later_patterns = [
        "not right now", "maybe later", "check back", "next quarter", "next year",
        "not a good time", "busy right now", "reach out later", "timing",
        "not in budget", "budget cycle", "revisit", "touch base later"
    ];
    
    for pattern in maybe_later_patterns {
        if text.contains(pattern) {
            return ("maybe_later".to_string(), 0.80);
        }
    }

    // Objection patterns (questions/concerns)
    let objection_patterns = [
        "how much", "what's the price", "pricing", "cost", "who else",
        "case study", "references", "competitors", "how does it work",
        "more information", "details", "what do you", "can you explain"
    ];
    
    for pattern in objection_patterns {
        if text.contains(pattern) {
            return ("objection".to_string(), 0.75);
        }
    }

    // Default to auto_reply with low confidence if nothing matches
    ("auto_reply".to_string(), 0.50)
}

pub async fn classify_reply_with_fallback(reply_text: &str) -> (String, f32) {
    // Try Claude first
    match classify_reply(reply_text).await {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!("Claude classification failed, using fallback: {}", e);
            classify_reply_simple(reply_text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_classification() {
        assert_eq!(classify_reply_simple("I'm out of office until Monday").0, "auto_reply");
        assert_eq!(classify_reply_simple("Please unsubscribe me").0, "negative");
        assert_eq!(classify_reply_simple("Let's schedule a call!").0, "interested");
        assert_eq!(classify_reply_simple("Not a good time, check back in Q2").0, "maybe_later");
        assert_eq!(classify_reply_simple("How much does this cost?").0, "objection");
    }
}
