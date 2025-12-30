use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    InvalidJson,
    NonExistentUser,
    DuplicateAction,
    MissingFields,
    InvalidTeamName,
    WrongType,
    WrongReason,
    FalsePositive,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::InvalidJson => write!(f, "Invalid JSON"),
            ErrorType::NonExistentUser => write!(f, "Non-existent user"),
            ErrorType::DuplicateAction => write!(f, "Duplicate action"),
            ErrorType::MissingFields => write!(f, "Missing required fields"),
            ErrorType::InvalidTeamName => write!(f, "Invalid team name"),
            ErrorType::WrongType => write!(f, "Wrong action type"),
            ErrorType::WrongReason => write!(f, "Wrong reason"),
            ErrorType::FalsePositive => write!(f, "False positive - user doesn't match filter"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub error_type: ErrorType,
    pub details: String,
}

impl ValidationError {
    pub fn new(error_type: ErrorType, details: impl Into<String>) -> Self {
        Self {
            error_type,
            details: details.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.details)
    }
}

impl std::error::Error for ValidationError {}
