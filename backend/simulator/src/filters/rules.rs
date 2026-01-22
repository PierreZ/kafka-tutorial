use super::{FilterMatch, TeamFilter};
use kafka_common::User;

/// Get the filter for a specific team number (1-15)
pub fn get_filter(team_number: u32) -> Box<dyn TeamFilter> {
    match team_number {
        1 => Box::new(Team1Filter),
        2 => Box::new(Team2Filter),
        3 => Box::new(Team3Filter),
        4 => Box::new(Team4Filter),
        5 => Box::new(Team5Filter),
        6 => Box::new(Team6Filter),
        7 => Box::new(Team7Filter),
        8 => Box::new(Team8Filter),
        9 => Box::new(Team9Filter),
        10 => Box::new(Team10Filter),
        11 => Box::new(Team11Filter),
        12 => Box::new(Team12Filter),
        13 => Box::new(Team13Filter),
        14 => Box::new(Team14Filter),
        15 => Box::new(Team15Filter),
        _ => panic!("Invalid team number: {}", team_number),
    }
}

// Team 1: email contains "hotmail"
pub struct Team1Filter;
impl TeamFilter for Team1Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.email.to_lowercase().contains("hotmail") {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "LEGACY_EMAIL_PROVIDER",
            })
        } else {
            None
        }
    }
}

// Team 2: credit_card_number starts with "37" (AMEX)
pub struct Team2Filter;
impl TeamFilter for Team2Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.credit_card_number.starts_with("37") {
            Some(FilterMatch {
                action_type: "CREDIT_CARD_VERIFICATION",
                reason: "AMEX_CARD",
            })
        } else {
            None
        }
    }
}

// Team 3: industry contains financial keywords
pub struct Team3Filter;
impl TeamFilter for Team3Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        let industry_lower = user.industry.to_lowercase();
        let keywords = ["bank", "financ", "insur", "invest", "account", "capital"];
        if keywords.iter().any(|k| industry_lower.contains(k)) {
            Some(FilterMatch {
                action_type: "SALES_LEAD",
                reason: "FINANCIAL_SECTOR",
            })
        } else {
            None
        }
    }
}

// Team 4: industry contains tech keywords
pub struct Team4Filter;
impl TeamFilter for Team4Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        let industry_lower = user.industry.to_lowercase();
        let keywords = [
            "computer",
            "internet",
            "semiconductor",
            "telecom",
            "wireless",
        ];
        if keywords.iter().any(|k| industry_lower.contains(k)) {
            Some(FilterMatch {
                action_type: "ACQUISITION_TARGET",
                reason: "TECH_COMPANY",
            })
        } else {
            None
        }
    }
}

// Team 5: industry contains healthcare keywords
pub struct Team5Filter;
impl TeamFilter for Team5Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        let industry_lower = user.industry.to_lowercase();
        let keywords = [
            "health",
            "hospital",
            "medical",
            "pharma",
            "biotech",
            "veterinary",
        ];
        if keywords.iter().any(|k| industry_lower.contains(k)) {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "HEALTHCARE_COMPLIANCE",
            })
        } else {
            None
        }
    }
}

// Team 6: profession equals "engineer"
pub struct Team6Filter;
impl TeamFilter for Team6Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.profession.to_lowercase() == "engineer" {
            Some(FilterMatch {
                action_type: "HIRE_CUSTOMER",
                reason: "IS_ENGINEER",
            })
        } else {
            None
        }
    }
}

// Team 7: premium AND credit > 10
pub struct Team7Filter;
impl TeamFilter for Team7Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.premium && user.credit > 10 {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "VIP_USER",
            })
        } else {
            None
        }
    }
}

// Team 8: credit < -15
pub struct Team8Filter;
impl TeamFilter for Team8Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.credit < -15 {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "CRITICAL_DEBT",
            })
        } else {
            None
        }
    }
}

// Team 9: time_zone starts with "Asia/"
pub struct Team9Filter;
impl TeamFilter for Team9Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.time_zone.starts_with("Asia/") {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "APAC_EXPANSION",
            })
        } else {
            None
        }
    }
}

// Team 10: user_agent contains "MSIE" or "Trident"
pub struct Team10Filter;
impl TeamFilter for Team10Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.user_agent.contains("MSIE") || user.user_agent.contains("Trident") {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "LEGACY_BROWSER",
            })
        } else {
            None
        }
    }
}

// Team 11: time_zone starts with "Europe/"
pub struct Team11Filter;
impl TeamFilter for Team11Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.time_zone.starts_with("Europe/") {
            Some(FilterMatch {
                action_type: "TRIGGER_GDPR_COMPLIANCE",
                reason: "IN_EUROPE",
            })
        } else {
            None
        }
    }
}

// Team 12: field equals "IT" or "Technology"
pub struct Team12Filter;
impl TeamFilter for Team12Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        let field_lower = user.field.to_lowercase();
        if field_lower == "it" || field_lower == "technology" {
            Some(FilterMatch {
                action_type: "HIRE_CUSTOMER",
                reason: "TECH_PROFESSIONAL",
            })
        } else {
            None
        }
    }
}

// Team 13: avatar contains "example.org" OR doesn't start with "https://"
pub struct Team13Filter;
impl TeamFilter for Team13Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.avatar.contains("example.org") || !user.avatar.starts_with("https://") {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "INVALID_AVATAR",
            })
        } else {
            None
        }
    }
}

// Team 14: name equals "John Doe"
pub struct Team14Filter;
impl TeamFilter for Team14Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.name == "John Doe" {
            Some(FilterMatch {
                action_type: "BAN_CUSTOMER",
                reason: "SUSPICIOUS_NAME",
            })
        } else {
            None
        }
    }
}

// Team 15: pack equals "free"
pub struct Team15Filter;
impl TeamFilter for Team15Filter {
    fn matches(&self, user: &User) -> Option<FilterMatch> {
        if user.pack == "free" {
            Some(FilterMatch {
                action_type: "CONTACT_CUSTOMER",
                reason: "UPGRADE_FREE",
            })
        } else {
            None
        }
    }
}
