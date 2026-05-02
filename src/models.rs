#[derive(Debug, PartialEq, Eq)]
pub enum QuestionError {
    IncompleteAction { remaining: usize },
}

impl std::fmt::Display for QuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompleteAction { remaining } => write!(
                f,
                "cannot mark question answered: {remaining} action(s) still incomplete"
            ),
        }
    }
}

impl std::error::Error for QuestionError {}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectiveError {
    UnansweredQuestion { remaining: usize },
}

impl std::fmt::Display for ObjectiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnansweredQuestion { remaining } => write!(
                f,
                "cannot mark objective met: {remaining} question(s) still unanswered"
            ),
        }
    }
}

impl std::error::Error for ObjectiveError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(String);

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionPriority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    LongTerm = 0,
}

impl QuestionPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::LongTerm => "long-term",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ActionPoint {
    content: String,
    category: ActionCategory,
    required_time: f32,
    completed: bool,
}

impl ActionPoint {
    pub fn new(content: String, category: ActionCategory, required_time: f32) -> Self {
        Self {
            content,
            category,
            required_time: required_time.max(0.0),
            completed: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn category(&self) -> &ActionCategory {
        &self.category
    }
    pub fn required_time(&self) -> f32 {
        self.required_time
    }
    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_category(&mut self, category: ActionCategory) {
        self.category = category;
    }
    pub fn set_required_time(&mut self, required_time: f32) {
        self.required_time = required_time.max(0.0);
    }
    pub fn set_completed(&mut self, completed: bool) {
        self.completed = completed;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    content: String,
    priority: QuestionPriority,
    actions: Vec<ActionPoint>,
    answered: bool,
}

impl Question {
    pub fn new(content: String, priority: QuestionPriority) -> Self {
        Self {
            content,
            priority,
            actions: Vec::new(),
            answered: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn priority(&self) -> &QuestionPriority {
        &self.priority
    }
    pub fn actions(&self) -> &[ActionPoint] {
        &self.actions
    }
    pub fn answered(&self) -> bool {
        self.answered
    }

    pub fn total_time_required(&self) -> f32 {
        self.actions.iter().map(|a| a.required_time).sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.actions
            .iter()
            .filter(|a| !a.completed)
            .map(|a| a.required_time)
            .sum()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_priority(&mut self, priority: QuestionPriority) {
        self.priority = priority;
    }
    pub fn set_answered(&mut self, answered: bool) -> Result<(), QuestionError> {
        if answered {
            let remaining = self.incomplete_action_count();
            if remaining > 0 {
                return Err(QuestionError::IncompleteAction { remaining });
            }
        }
        self.answered = answered;
        Ok(())
    }

    pub fn push_action(&mut self, action: ActionPoint) {
        self.actions.push(action);
    }
    pub fn remove_action(&mut self, idx: usize) -> ActionPoint {
        self.actions.remove(idx)
    }
    pub fn action_mut(&mut self, idx: usize) -> Option<&mut ActionPoint> {
        self.actions.get_mut(idx)
    }

    pub fn incomplete_action_count(&self) -> usize {
        self.actions.iter().filter(|a| !a.completed).count()
    }
    pub fn total_action_count(&self) -> usize {
        self.actions.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Objective {
    content: String,
    questions: Vec<Question>,
    tags: Vec<Tag>,
    met: bool,
}

impl Objective {
    pub fn new(content: String) -> Self {
        Self {
            content,
            questions: Vec::new(),
            tags: Vec::new(),
            met: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    pub fn met(&self) -> bool {
        self.met
    }

    pub fn has_tag(&self, name: &str) -> bool {
        self.tags.iter().any(|t| t.name() == name)
    }
    pub fn add_tag(&mut self, tag: Tag) -> bool {
        if self.has_tag(tag.name()) {
            return false;
        }
        self.tags.push(tag);
        true
    }
    pub fn remove_tag(&mut self, name: &str) -> Option<Tag> {
        let idx = self.tags.iter().position(|t| t.name() == name)?;
        Some(self.tags.remove(idx))
    }

    pub fn total_allocated_timeframe(&self) -> f32 {
        self.questions.iter().map(|q| q.total_time_required()).sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.questions
            .iter()
            .filter(|q| !q.answered())
            .map(|q| q.remaining_time_needed())
            .sum()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_met(&mut self, met: bool) -> Result<(), ObjectiveError> {
        if met {
            let remaining = self.unanswered_question_count();
            if remaining > 0 {
                return Err(ObjectiveError::UnansweredQuestion { remaining });
            }
        }
        self.met = met;
        Ok(())
    }

    pub fn push_question(&mut self, question: Question) {
        self.questions.push(question);
    }
    pub fn remove_question(&mut self, idx: usize) -> Question {
        self.questions.remove(idx)
    }
    pub fn question_mut(&mut self, idx: usize) -> Option<&mut Question> {
        self.questions.get_mut(idx)
    }

    pub fn unanswered_question_count(&self) -> usize {
        self.questions.iter().filter(|q| !q.answered()).count()
    }
    pub fn total_question_count(&self) -> usize {
        self.questions.len()
    }
}
