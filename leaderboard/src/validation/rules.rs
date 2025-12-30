use serde::Deserialize;

/// Simplified action struct - just for parsing and extracting team
#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub customer: Option<String>,
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub reason: Option<String>,
    pub team: String,
}

/// Simple validation result - no cross-consumption validation
#[derive(Debug, Clone)]
pub enum SimpleValidationResult {
    /// Valid action with team name
    Valid { team: String },
    /// Invalid JSON (can't parse at all)
    InvalidJson,
    /// Missing required fields (parsed but incomplete)
    MissingFields { team: String },
}

/// Minimal validation - no cross-consumption of new_users
/// Just checks: valid JSON and has required fields
pub fn validate_action_simple(payload: &[u8]) -> SimpleValidationResult {
    // Step 1: Parse JSON
    let action: Action = match serde_json::from_slice(payload) {
        Ok(a) => a,
        Err(_) => return SimpleValidationResult::InvalidJson,
    };

    // Step 2: Check required fields exist and are non-empty
    let has_customer = action
        .customer
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let has_type = action
        .action_type
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let has_reason = action
        .reason
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let has_team = !action.team.is_empty();

    if !has_customer || !has_type || !has_reason || !has_team {
        return SimpleValidationResult::MissingFields { team: action.team };
    }

    SimpleValidationResult::Valid { team: action.team }
}

/// Extract team name from payload if possible (for error reporting)
pub fn extract_team_from_payload(payload: &[u8]) -> Option<String> {
    #[derive(Deserialize)]
    struct TeamOnly {
        team: Option<String>,
    }

    serde_json::from_slice::<TeamOnly>(payload)
        .ok()
        .and_then(|t| t.team)
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_action() {
        let payload = br#"{"customer":"user1","type":"CONTACT","reason":"VIP","team":"team-1"}"#;
        match validate_action_simple(payload) {
            SimpleValidationResult::Valid { team } => assert_eq!(team, "team-1"),
            _ => panic!("Expected Valid"),
        }
    }

    #[test]
    fn test_invalid_json() {
        let payload = b"not json";
        assert!(matches!(
            validate_action_simple(payload),
            SimpleValidationResult::InvalidJson
        ));
    }

    #[test]
    fn test_missing_fields() {
        let payload = br#"{"team":"team-1"}"#;
        match validate_action_simple(payload) {
            SimpleValidationResult::MissingFields { team } => assert_eq!(team, "team-1"),
            _ => panic!("Expected MissingFields"),
        }
    }

    #[test]
    fn test_empty_fields() {
        let payload = br#"{"customer":"","type":"CONTACT","reason":"VIP","team":"team-1"}"#;
        match validate_action_simple(payload) {
            SimpleValidationResult::MissingFields { team } => assert_eq!(team, "team-1"),
            _ => panic!("Expected MissingFields for empty customer"),
        }
    }

    #[test]
    fn test_extract_team() {
        let payload = br#"{"team":"team-5","other":"data"}"#;
        assert_eq!(
            extract_team_from_payload(payload),
            Some("team-5".to_string())
        );

        let payload = b"invalid";
        assert_eq!(extract_team_from_payload(payload), None);
    }
}
