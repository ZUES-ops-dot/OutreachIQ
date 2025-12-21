use crate::models::lead::VerificationStatus;
use reqwest::Client;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedLead {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub linkedin_url: Option<String>,
    pub verification_status: VerificationStatus,
    pub confidence_score: f32,
    pub signals: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub verified_at: Option<chrono::DateTime<Utc>>,
}

pub struct LeadGenerator {
    client: Client,
}

impl LeadGenerator {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Generate leads based on signals and vertical/industry
    pub async fn generate_leads(
        &self,
        vertical: &str,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let leads = match vertical.to_lowercase().as_str() {
            "web3" | "crypto" | "blockchain" => self.generate_web3_leads(role, limit).await?,
            "saas" | "software" | "tech" => self.generate_saas_leads(role, limit).await?,
            "agency" | "marketing" | "creative" => self.generate_agency_leads(role, limit).await?,
            "ecommerce" | "retail" | "commerce" => self.generate_ecommerce_leads(role, limit).await?,
            "fintech" | "finance" | "banking" => self.generate_fintech_leads(role, limit).await?,
            "healthcare" | "health" | "medical" => self.generate_healthcare_leads(role, limit).await?,
            "education" | "edtech" | "learning" => self.generate_education_leads(role, limit).await?,
            "real_estate" | "realestate" | "property" => self.generate_realestate_leads(role, limit).await?,
            "consulting" | "professional_services" => self.generate_consulting_leads(role, limit).await?,
            "manufacturing" | "industrial" => self.generate_manufacturing_leads(role, limit).await?,
            "media" | "entertainment" | "content" => self.generate_media_leads(role, limit).await?,
            "logistics" | "supply_chain" | "transportation" => self.generate_logistics_leads(role, limit).await?,
            "all" | "any" | "" => self.generate_mixed_leads(role, limit).await?,
            _ => self.generate_custom_leads(vertical, role, limit).await?,
        };

        Ok(leads)
    }

    async fn generate_web3_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("Ethereum Labs", "ethereumlabs.io"),
            ("DeFi Protocol", "defiprotocol.xyz"),
            ("NFT Marketplace", "nftmarket.io"),
            ("Chain Analytics", "chainanalytics.co"),
            ("Web3 Wallet", "web3wallet.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(format!("{} {}", company, i / companies.len())),
                title: role.map(|r| r.to_string()).or(Some("Founder".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.75,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "funding_event": "Pre-Seed",
                    "tech_stack": ["Solidity", "React", "Node.js"],
                    "growth_indicators": ["Active GitHub", "Recent launch"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_saas_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("CloudSync", "cloudsync.io"),
            ("DataFlow", "dataflow.com"),
            ("MetricsPro", "metricspro.io"),
            ("AutomateHQ", "automatehq.com"),
            ("ScaleUp", "scaleup.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(format!("{} Inc", company)),
                title: role.map(|r| r.to_string()).or(Some("CEO".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.85,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Stripe", "AWS", "PostgreSQL"],
                    "company_size": "10-50",
                    "growth_indicators": ["Series A funding", "Hiring sales team"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_agency_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("Digital Spark", "digitalspark.co"),
            ("Creative Labs", "creativelabs.agency"),
            ("Growth Studio", "growthstudio.io"),
            ("Brand Forge", "brandforge.co"),
            ("Pixel Perfect", "pixelperfect.agency"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Creative Director".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.80,
                signals: serde_json::json!({
                    "recent_hiring": false,
                    "tech_stack": ["Webflow", "WordPress", "Figma"],
                    "company_size": "5-20",
                    "growth_indicators": ["New client case studies"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    fn generate_first_name(&self, index: usize) -> String {
        let names = vec![
            "Alex", "Jordan", "Taylor", "Morgan", "Casey",
            "Riley", "Quinn", "Avery", "Parker", "Drew",
            "Blake", "Cameron", "Dakota", "Emery", "Finley",
        ];
        names[index % names.len()].to_string()
    }

    fn generate_last_name(&self, index: usize) -> String {
        let names = vec![
            "Smith", "Johnson", "Williams", "Brown", "Jones",
            "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
            "Anderson", "Taylor", "Thomas", "Moore", "Jackson",
        ];
        names[index % names.len()].to_string()
    }

    fn generate_email_pattern(&self, first_name: &str, last_name: &str, domain: &str) -> String {
        format!("{}.{}@{}", first_name.to_lowercase(), last_name.to_lowercase(), domain)
    }

    async fn generate_ecommerce_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("ShopifyPlus Store", "shopifyplus.store"),
            ("Commerce Cloud", "commercecloud.io"),
            ("RetailTech", "retailtech.com"),
            ("CartGenius", "cartgenius.io"),
            ("Fulfillment Pro", "fulfillmentpro.com"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("E-commerce Manager".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.82,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Shopify", "Klaviyo", "Stripe"],
                    "company_size": "20-100",
                    "growth_indicators": ["Expanding product lines", "New warehouse"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_fintech_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("PayFlow", "payflow.io"),
            ("LendTech", "lendtech.com"),
            ("InvestPro", "investpro.io"),
            ("BankingAPI", "bankingapi.com"),
            ("WealthStack", "wealthstack.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Head of Product".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.88,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "funding_event": "Series B",
                    "tech_stack": ["Plaid", "Stripe", "AWS"],
                    "growth_indicators": ["New banking license", "International expansion"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_healthcare_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("HealthTech Solutions", "healthtech.io"),
            ("MedConnect", "medconnect.com"),
            ("CareFlow", "careflow.health"),
            ("TeleDoc Pro", "teledocpro.com"),
            ("PharmaTech", "pharmatech.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Chief Medical Officer".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.80,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Epic", "Salesforce Health", "AWS"],
                    "company_size": "50-200",
                    "growth_indicators": ["HIPAA certified", "New clinic locations"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_education_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("LearnHub", "learnhub.io"),
            ("EdTech Academy", "edtechacademy.com"),
            ("SkillPath", "skillpath.io"),
            ("CourseBuilder", "coursebuilder.com"),
            ("TutorTech", "tutortech.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Director of Learning".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.78,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Canvas", "Zoom", "Notion"],
                    "company_size": "10-50",
                    "growth_indicators": ["New course launches", "B2B partnerships"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_realestate_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("PropTech Solutions", "proptech.io"),
            ("RealtyFlow", "realtyflow.com"),
            ("HomeStack", "homestack.io"),
            ("PropertyPro", "propertypro.com"),
            ("EstateManager", "estatemanager.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Managing Broker".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.76,
                signals: serde_json::json!({
                    "recent_hiring": false,
                    "tech_stack": ["Zillow API", "Salesforce", "DocuSign"],
                    "company_size": "20-100",
                    "growth_indicators": ["New market expansion", "Tech adoption"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_consulting_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("Strategy Partners", "strategypartners.co"),
            ("Growth Advisors", "growthadvisors.com"),
            ("Digital Consulting", "digitalconsulting.io"),
            ("Transform Group", "transformgroup.co"),
            ("Innovation Labs", "innovationlabs.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Managing Partner".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.83,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Notion", "Slack", "Monday.com"],
                    "company_size": "10-50",
                    "growth_indicators": ["New practice areas", "Partner promotions"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_manufacturing_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("SmartFactory", "smartfactory.io"),
            ("Industrial IoT", "industrialiot.com"),
            ("ManuTech", "manutech.io"),
            ("AutomationPro", "automationpro.com"),
            ("SupplyChain Tech", "supplychaintech.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("VP Operations".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.79,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["SAP", "Siemens", "Azure IoT"],
                    "company_size": "100-500",
                    "growth_indicators": ["New facility", "Automation investment"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_media_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("ContentHub", "contenthub.io"),
            ("MediaStream", "mediastream.com"),
            ("CreatorPlatform", "creatorplatform.io"),
            ("VideoTech", "videotech.com"),
            ("PodcastPro", "podcastpro.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Head of Content".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.81,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Adobe", "Vimeo", "Spotify"],
                    "company_size": "10-50",
                    "growth_indicators": ["New show launches", "Sponsorship deals"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_logistics_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        let mut leads = Vec::new();
        let companies = vec![
            ("FleetManager", "fleetmanager.io"),
            ("LogiTech", "logitech-supply.com"),
            ("ShipFast", "shipfast.io"),
            ("WarehousePro", "warehousepro.com"),
            ("RouteOptimize", "routeoptimize.io"),
        ];

        for i in 0..limit {
            let (company, domain) = &companies[i % companies.len()];
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(company.to_string()),
                title: role.map(|r| r.to_string()).or(Some("Logistics Director".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.77,
                signals: serde_json::json!({
                    "recent_hiring": true,
                    "tech_stack": ["Oracle", "SAP", "GPS Fleet"],
                    "company_size": "50-200",
                    "growth_indicators": ["Fleet expansion", "New routes"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }

    async fn generate_mixed_leads(
        &self,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        // Generate leads from multiple industries
        let mut all_leads = Vec::new();
        let per_industry = (limit / 5).max(1);
        
        all_leads.extend(self.generate_saas_leads(role, per_industry).await?);
        all_leads.extend(self.generate_ecommerce_leads(role, per_industry).await?);
        all_leads.extend(self.generate_fintech_leads(role, per_industry).await?);
        all_leads.extend(self.generate_healthcare_leads(role, per_industry).await?);
        all_leads.extend(self.generate_consulting_leads(role, per_industry).await?);
        
        // Trim to exact limit
        all_leads.truncate(limit);
        Ok(all_leads)
    }

    async fn generate_custom_leads(
        &self,
        industry: &str,
        role: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GeneratedLead>, Box<dyn std::error::Error + Send + Sync>> {
        // Generic lead generation for any custom industry
        let mut leads = Vec::new();
        let domain = format!("{}.com", industry.to_lowercase().replace(' ', ""));
        
        for i in 0..limit {
            let first_name = self.generate_first_name(i);
            let last_name = self.generate_last_name(i);
            
            let lead = GeneratedLead {
                id: Uuid::new_v4(),
                email: self.generate_email_pattern(&first_name, &last_name, &domain),
                first_name: Some(first_name),
                last_name: Some(last_name),
                company: Some(format!("{} Company {}", industry, i + 1)),
                title: role.map(|r| r.to_string()).or(Some("Decision Maker".to_string())),
                linkedin_url: None,
                verification_status: VerificationStatus::Pending,
                confidence_score: 0.70,
                signals: serde_json::json!({
                    "industry": industry,
                    "recent_hiring": false,
                    "growth_indicators": ["Active online presence"]
                }),
                created_at: Utc::now(),
                verified_at: None,
            };
            leads.push(lead);
        }

        Ok(leads)
    }
}

impl Default for LeadGenerator {
    fn default() -> Self {
        Self::new()
    }
}
