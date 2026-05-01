pub struct Objective {
    content: String,
    questions: Vec<Question>,
    met: bool,
}

impl Objective {
    pub fn new(content: String) -> Self {
        Self {
            content,
            questions: Vec::new(),
            met: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }
    pub fn met(&self) -> bool {
        self.met
    }

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

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_met(&mut self, met: bool) {
        self.met = met;
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
}

pub enum QuestionPriority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    LongTerm = 0,
}

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
        self.actions.iter().map(|action| action.required_time).sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.actions
            .iter()
            .filter(|action| !action.completed)
            .map(|action| action.required_time)
            .sum()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_priority(&mut self, priority: QuestionPriority) {
        self.priority = priority;
    }
    pub fn set_answered(&mut self, answered: bool) {
        self.answered = answered;
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
