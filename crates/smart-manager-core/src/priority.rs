use crate::questions::QuestionPriority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    LongTerm,
}

impl Priority {
    pub fn score(self) -> u8 {
        match self {
            Self::Critical => 4,
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
            Self::LongTerm => 0,
        }
    }
    pub fn letter(self) -> char {
        match self {
            Self::Critical => 'C',
            Self::High => 'H',
            Self::Medium => 'M',
            Self::Low => 'L',
            Self::LongTerm => 'T',
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::LongTerm => "LongTerm",
        }
    }
}

impl From<&QuestionPriority> for Priority {
    fn from(p: &QuestionPriority) -> Self {
        match p {
            QuestionPriority::Critical => Self::Critical,
            QuestionPriority::High => Self::High,
            QuestionPriority::Medium => Self::Medium,
            QuestionPriority::Low => Self::Low,
            QuestionPriority::LongTerm => Self::LongTerm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_orders_descending_from_critical() {
        assert!(Priority::Critical.score() > Priority::High.score());
        assert!(Priority::High.score() > Priority::Medium.score());
        assert!(Priority::Medium.score() > Priority::Low.score());
        assert!(Priority::Low.score() > Priority::LongTerm.score());
    }

    #[test]
    fn test_letter_returns_first_letter_for_each_variant() {
        assert_eq!(Priority::Critical.letter(), 'C');
        assert_eq!(Priority::High.letter(), 'H');
        assert_eq!(Priority::Medium.letter(), 'M');
        assert_eq!(Priority::Low.letter(), 'L');
        assert_eq!(Priority::LongTerm.letter(), 'T');
    }

    #[test]
    fn test_from_question_priority_maps_each_variant() {
        assert_eq!(
            Priority::from(&QuestionPriority::Critical),
            Priority::Critical
        );
        assert_eq!(Priority::from(&QuestionPriority::High), Priority::High);
        assert_eq!(Priority::from(&QuestionPriority::Medium), Priority::Medium);
        assert_eq!(Priority::from(&QuestionPriority::Low), Priority::Low);
        assert_eq!(
            Priority::from(&QuestionPriority::LongTerm),
            Priority::LongTerm
        );
    }
}
