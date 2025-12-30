use super::errors::{ErrorType, ValidationError};
use serde::{Deserialize, Serialize};

/// User struct matching the producer's User struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub credit_card_number: String,
    pub company_name: String,
    pub company_slogan: String,
    pub industry: String,
    pub user_name: String,
    pub avatar: String,
    pub name: String,
    pub profession: String,
    pub field: String,
    pub premium: bool,
    pub credit: i32,
    pub time_zone: String,
    pub user_agent: String,
    pub pack: String,
}

/// Action struct produced by students
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub customer: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub reason: String,
    pub team: String,
}

/// Expected action output for each team
#[derive(Debug, Clone)]
pub struct ExpectedAction {
    pub action_type: &'static str,
    pub reason: &'static str,
}

/// Team validation rules (1-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRule {
    Team1,
    Team2,
    Team3,
    Team4,
    Team5,
    Team6,
    Team7,
    Team8,
    Team9,
    Team10,
    Team11,
    Team12,
    Team13,
    Team14,
    Team15,
}

impl TeamRule {
    /// Parse team name (e.g., "team-1") into TeamRule
    pub fn from_team_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "team-1" => Some(TeamRule::Team1),
            "team-2" => Some(TeamRule::Team2),
            "team-3" => Some(TeamRule::Team3),
            "team-4" => Some(TeamRule::Team4),
            "team-5" => Some(TeamRule::Team5),
            "team-6" => Some(TeamRule::Team6),
            "team-7" => Some(TeamRule::Team7),
            "team-8" => Some(TeamRule::Team8),
            "team-9" => Some(TeamRule::Team9),
            "team-10" => Some(TeamRule::Team10),
            "team-11" => Some(TeamRule::Team11),
            "team-12" => Some(TeamRule::Team12),
            "team-13" => Some(TeamRule::Team13),
            "team-14" => Some(TeamRule::Team14),
            "team-15" => Some(TeamRule::Team15),
            _ => None,
        }
    }

    /// Check if a user matches this team's filter criteria
    pub fn matches(&self, user: &User) -> bool {
        match self {
            // Team-1: email contains "hotmail"
            TeamRule::Team1 => user.email.to_lowercase().contains("hotmail"),

            // Team-2: credit_card_number starts with "37"
            TeamRule::Team2 => user.credit_card_number.starts_with("37"),

            // Team-3: industry contains Bank/Financ/Insur/Invest/Account/Capital
            TeamRule::Team3 => {
                let industry = user.industry.to_lowercase();
                industry.contains("bank")
                    || industry.contains("financ")
                    || industry.contains("insur")
                    || industry.contains("invest")
                    || industry.contains("account")
                    || industry.contains("capital")
            }

            // Team-4: industry contains Computer/Internet/Semiconductor/Telecom/Wireless
            TeamRule::Team4 => {
                let industry = user.industry.to_lowercase();
                industry.contains("computer")
                    || industry.contains("internet")
                    || industry.contains("semiconductor")
                    || industry.contains("telecom")
                    || industry.contains("wireless")
            }

            // Team-5: industry contains Health/Hospital/Medical/Pharma/Biotech/Veterinary
            TeamRule::Team5 => {
                let industry = user.industry.to_lowercase();
                industry.contains("health")
                    || industry.contains("hospital")
                    || industry.contains("medical")
                    || industry.contains("pharma")
                    || industry.contains("biotech")
                    || industry.contains("veterinary")
            }

            // Team-6: profession is "engineer"
            TeamRule::Team6 => user.profession.to_lowercase() == "engineer",

            // Team-7: premium == true AND credit > 10
            TeamRule::Team7 => user.premium && user.credit > 10,

            // Team-8: credit < -15
            TeamRule::Team8 => user.credit < -15,

            // Team-9: time_zone starts with "Asia/"
            TeamRule::Team9 => user.time_zone.starts_with("Asia/"),

            // Team-10: user_agent contains "MSIE" or "Trident"
            TeamRule::Team10 => {
                user.user_agent.contains("MSIE") || user.user_agent.contains("Trident")
            }

            // Team-11: time_zone starts with "Europe/"
            TeamRule::Team11 => user.time_zone.starts_with("Europe/"),

            // Team-12: field is "IT" or "Technology"
            TeamRule::Team12 => {
                let field = user.field.to_lowercase();
                field == "it" || field == "technology"
            }

            // Team-13: avatar is "example.org" OR doesn't start with "https://"
            TeamRule::Team13 => {
                user.avatar == "example.org" || !user.avatar.starts_with("https://")
            }

            // Team-14: name is "John Doe"
            TeamRule::Team14 => user.name == "John Doe",

            // Team-15: pack is "free"
            TeamRule::Team15 => user.pack.to_lowercase() == "free",
        }
    }

    /// Get the expected action type and reason for this team
    pub fn expected_action(&self) -> ExpectedAction {
        match self {
            TeamRule::Team1 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "LEGACY_EMAIL_PROVIDER",
            },
            TeamRule::Team2 => ExpectedAction {
                action_type: "CREDIT_CARD_VERIFICATION",
                reason: "AMEX_CARD",
            },
            TeamRule::Team3 => ExpectedAction {
                action_type: "SALES_LEAD",
                reason: "FINANCIAL_SECTOR",
            },
            TeamRule::Team4 => ExpectedAction {
                action_type: "ACQUISITION_TARGET",
                reason: "TECH_COMPANY",
            },
            TeamRule::Team5 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "HEALTHCARE_COMPLIANCE",
            },
            TeamRule::Team6 => ExpectedAction {
                action_type: "HIRE_CUSTOMER",
                reason: "IS_ENGINEER",
            },
            TeamRule::Team7 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "VIP_USER",
            },
            TeamRule::Team8 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "CRITICAL_DEBT",
            },
            TeamRule::Team9 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "APAC_EXPANSION",
            },
            TeamRule::Team10 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "LEGACY_BROWSER",
            },
            TeamRule::Team11 => ExpectedAction {
                action_type: "TRIGGER_GDPR_COMPLIANCE",
                reason: "IN_EUROPE",
            },
            TeamRule::Team12 => ExpectedAction {
                action_type: "HIRE_CUSTOMER",
                reason: "TECH_PROFESSIONAL",
            },
            TeamRule::Team13 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "INVALID_AVATAR",
            },
            TeamRule::Team14 => ExpectedAction {
                action_type: "BAN_CUSTOMER",
                reason: "SUSPICIOUS_NAME",
            },
            TeamRule::Team15 => ExpectedAction {
                action_type: "CONTACT_CUSTOMER",
                reason: "UPGRADE_FREE",
            },
        }
    }
}

/// Result of validating an action
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Correct { points: u32 },
    Incorrect { error: ValidationError },
}

/// Validate an action against a user and team rules
pub fn validate_action(
    action: &Action,
    user: Option<&User>,
    processed_keys: &std::collections::HashSet<(String, String)>,
    points_per_correct: u32,
) -> ValidationResult {
    // Check required fields
    if action.customer.is_empty()
        || action.action_type.is_empty()
        || action.reason.is_empty()
        || action.team.is_empty()
    {
        return ValidationResult::Incorrect {
            error: ValidationError::new(
                ErrorType::MissingFields,
                "Action missing required fields (customer, type, reason, team)",
            ),
        };
    }

    // Check team name validity
    let rule = match TeamRule::from_team_name(&action.team) {
        Some(r) => r,
        None => {
            return ValidationResult::Incorrect {
                error: ValidationError::new(
                    ErrorType::InvalidTeamName,
                    format!("Unknown team: {}", action.team),
                ),
            }
        }
    };

    // Check for duplicate
    let key = (action.customer.clone(), action.team.clone());
    if processed_keys.contains(&key) {
        return ValidationResult::Incorrect {
            error: ValidationError::new(
                ErrorType::DuplicateAction,
                format!(
                    "Duplicate action for customer {} by {}",
                    action.customer, action.team
                ),
            ),
        };
    }

    // Lookup user
    let user = match user {
        Some(u) => u,
        None => {
            return ValidationResult::Incorrect {
                error: ValidationError::new(
                    ErrorType::NonExistentUser,
                    format!("User not found: {}", action.customer),
                ),
            }
        }
    };

    // Verify the user matches the team's rule (check for false positive)
    if !rule.matches(user) {
        return ValidationResult::Incorrect {
            error: ValidationError::new(
                ErrorType::FalsePositive,
                format!(
                    "User {} does not match {}'s filter criteria",
                    action.customer, action.team
                ),
            ),
        };
    }

    // Verify type and reason
    let expected = rule.expected_action();
    if action.action_type != expected.action_type {
        return ValidationResult::Incorrect {
            error: ValidationError::new(
                ErrorType::WrongType,
                format!(
                    "Expected type '{}', got '{}'",
                    expected.action_type, action.action_type
                ),
            ),
        };
    }
    if action.reason != expected.reason {
        return ValidationResult::Incorrect {
            error: ValidationError::new(
                ErrorType::WrongReason,
                format!(
                    "Expected reason '{}', got '{}'",
                    expected.reason, action.reason
                ),
            ),
        };
    }

    ValidationResult::Correct {
        points: points_per_correct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User {
            email: "test@hotmail.com".to_string(),
            credit_card_number: "3712345678901234".to_string(),
            company_name: "Test Corp".to_string(),
            company_slogan: "Testing is our passion".to_string(),
            industry: "Banking".to_string(),
            user_name: "testuser".to_string(),
            avatar: "https://robohash.org/test.png".to_string(),
            name: "Test User".to_string(),
            profession: "engineer".to_string(),
            field: "IT".to_string(),
            premium: true,
            credit: 15,
            time_zone: "Europe/Paris".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            pack: "free".to_string(),
        }
    }

    #[test]
    fn test_team1_matches_hotmail() {
        let mut user = create_test_user();
        user.email = "user@hotmail.com".to_string();
        assert!(TeamRule::Team1.matches(&user));

        user.email = "user@gmail.com".to_string();
        assert!(!TeamRule::Team1.matches(&user));
    }

    #[test]
    fn test_team2_matches_amex() {
        let mut user = create_test_user();
        user.credit_card_number = "371234567890".to_string();
        assert!(TeamRule::Team2.matches(&user));

        user.credit_card_number = "411234567890".to_string();
        assert!(!TeamRule::Team2.matches(&user));
    }

    #[test]
    fn test_team6_matches_engineer() {
        let mut user = create_test_user();
        user.profession = "engineer".to_string();
        assert!(TeamRule::Team6.matches(&user));

        user.profession = "developer".to_string();
        assert!(!TeamRule::Team6.matches(&user));
    }

    #[test]
    fn test_team7_matches_vip() {
        let mut user = create_test_user();
        user.premium = true;
        user.credit = 15;
        assert!(TeamRule::Team7.matches(&user));

        user.credit = 5;
        assert!(!TeamRule::Team7.matches(&user));

        user.credit = 15;
        user.premium = false;
        assert!(!TeamRule::Team7.matches(&user));
    }

    #[test]
    fn test_team14_matches_john_doe() {
        let mut user = create_test_user();
        user.name = "John Doe".to_string();
        assert!(TeamRule::Team14.matches(&user));

        user.name = "Jane Doe".to_string();
        assert!(!TeamRule::Team14.matches(&user));
    }

    #[test]
    fn test_expected_actions() {
        assert_eq!(TeamRule::Team1.expected_action().action_type, "CONTACT_CUSTOMER");
        assert_eq!(TeamRule::Team1.expected_action().reason, "LEGACY_EMAIL_PROVIDER");

        assert_eq!(TeamRule::Team2.expected_action().action_type, "CREDIT_CARD_VERIFICATION");
        assert_eq!(TeamRule::Team2.expected_action().reason, "AMEX_CARD");
    }
}
