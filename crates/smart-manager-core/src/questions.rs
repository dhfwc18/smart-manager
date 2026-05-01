pub struct Objective {
    pub content: String,
    pub questions: Vec<Question>,
    pub met: bool,
}

impl Objective {
    pub fn total_allocated_timeframe(&self) -> f32 {
        self.questions
            .iter()
            .map(|question| question.total_time_required())
            .sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.questions
            .iter()
            .map(|question| question.remaining_time_needed())
            .sum()
    }
}

pub enum QuestionPriority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    LongTerm = 0,
}

pub struct Question {
    pub content: String,
    pub priority: QuestionPriority,
    pub actions: Vec<ActionPoint>,
    pub answered: bool,
}

impl Question {
    pub fn total_time_required(&self) -> f32 {
        self.actions.iter().map(|action| action.required_time).sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.actions
            .iter()
            .filter(|action| !action.completed)
            .map(|action| action.required_time)
            .sum()
    }
}

pub enum ActionCategory {
    Writing,
    Managing,
    Qa,
    Analysis,
    Research,
    Programming,
    Presentation,
}

impl ActionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Writing => "writing",
            Self::Managing => "managing",
            Self::Qa => "qa",
            Self::Analysis => "analysis",
            Self::Research => "research",
            Self::Programming => "programming",
            Self::Presentation => "presentation",
        }
    }
}

pub struct ActionPoint {
    pub text: String,
    pub category: ActionCategory,
    pub required_time: f32,
    pub completed: bool,
}
