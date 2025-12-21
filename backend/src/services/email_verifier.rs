use crate::models::lead::VerificationStatus;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use regex::Regex;

pub struct EmailVerifier {
    resolver: TokioAsyncResolver,
}

impl EmailVerifier {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );
        Ok(Self { resolver })
    }

    pub async fn verify_email(&self, email: &str) -> (VerificationStatus, f32) {
        // Syntax check
        if !self.is_valid_syntax(email) {
            return (VerificationStatus::Invalid, 0.0);
        }

        // Extract domain
        let domain = match email.split('@').nth(1) {
            Some(d) => d,
            None => return (VerificationStatus::Invalid, 0.0),
        };
        
        // MX record check
        let mx_valid = self.check_mx_record(domain).await;
        if !mx_valid {
            return (VerificationStatus::Invalid, 0.2);
        }

        // Disposable email check
        if self.is_disposable_domain(domain) {
            return (VerificationStatus::Risky, 0.3);
        }

        // Role-based email check
        let local = email.split('@').next().unwrap_or("");
        if self.is_role_based(local) {
            return (VerificationStatus::Risky, 0.4);
        }

        // Calculate confidence score
        let confidence = self.calculate_confidence(email, domain);

        if confidence > 0.7 {
            (VerificationStatus::Valid, confidence)
        } else if confidence > 0.4 {
            (VerificationStatus::Risky, confidence)
        } else {
            (VerificationStatus::Invalid, confidence)
        }
    }

    pub async fn verify_batch(&self, emails: &[String]) -> Vec<(String, VerificationStatus, f32)> {
        let mut results = Vec::new();
        for email in emails {
            let (status, confidence) = self.verify_email(email).await;
            results.push((email.clone(), status, confidence));
        }
        results
    }

    fn is_valid_syntax(&self, email: &str) -> bool {
        let email_regex = Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();
        email_regex.is_match(email)
    }

    async fn check_mx_record(&self, domain: &str) -> bool {
        match self.resolver.mx_lookup(domain).await {
            Ok(mx_records) => mx_records.iter().next().is_some(),
            Err(_) => false,
        }
    }

    fn is_disposable_domain(&self, domain: &str) -> bool {
        let disposable_domains = vec![
            "tempmail.com", "guerrillamail.com", "10minutemail.com",
            "mailinator.com", "throwaway.email", "temp-mail.org",
            "fakeinbox.com", "trashmail.com", "yopmail.com",
            "sharklasers.com", "guerrillamail.info", "grr.la",
        ];
        disposable_domains.iter().any(|d| domain.contains(d))
    }

    fn is_role_based(&self, local: &str) -> bool {
        let role_based = vec![
            "info", "contact", "support", "sales", "admin",
            "help", "billing", "noreply", "no-reply", "webmaster",
            "postmaster", "hostmaster", "abuse", "security",
        ];
        role_based.iter().any(|r| local.to_lowercase().starts_with(r))
    }

    fn calculate_confidence(&self, email: &str, domain: &str) -> f32 {
        let mut score: f32 = 0.5;

        // Domain has good TLD
        if domain.ends_with(".com") || domain.ends_with(".io") || domain.ends_with(".co") {
            score += 0.15;
        }

        // Email follows common patterns (first.last@domain)
        let local = email.split('@').next().unwrap_or("");
        if local.contains('.') {
            score += 0.15;
        }
        
        // Reasonable length
        if local.len() >= 5 && local.len() <= 30 {
            score += 0.1;
        }

        // Not a generic address
        let generic = vec!["info", "contact", "support", "sales", "admin"];
        if !generic.iter().any(|g| local.starts_with(g)) {
            score += 0.1;
        }

        score.min(1.0)
    }
}
