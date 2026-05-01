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
            .filter(|question| !question.answered())
            .map(|question| question.remaining_time_needed())
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

    pub fn unanswered_question_count(&mut self) -> usize {
        self.questions
            .iter()
            .filter(|question| !question.answered())
            .count()
    }

    pub fn total_question_count(&mut self) -> usize {
        self.questions.len()
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

    pub fn incomplete_action_count(&mut self) -> usize {
        self.actions
            .iter()
            .filter(|action| !action.completed)
            .count()
    }

    pub fn total_action_count(&mut self) -> usize {
        self.actions.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn action(time: f32, completed: bool) -> ActionPoint {
        let mut a = ActionPoint::new("a".to_string(), ActionCategory::Writing, time);
        a.set_completed(completed);
        a
    }

    fn question_with(actions: Vec<ActionPoint>, answered: bool) -> Question {
        let mut q = Question::new("q".to_string(), QuestionPriority::Medium);
        for a in actions {
            q.push_action(a);
        }
        q.set_answered(answered)
            .expect("test setup: incomplete actions");
        q
    }

    // ActionCategory

    #[test]
    fn test_as_str_writing_returns_writing() {
        assert_eq!(ActionCategory::Writing.as_str(), "writing");
    }

    #[test]
    fn test_as_str_presentation_returns_presentation() {
        assert_eq!(ActionCategory::Presentation.as_str(), "presentation");
    }

    // ActionPoint

    #[test]
    fn test_new_action_with_valid_time_stores_value() {
        let a = ActionPoint::new("write".into(), ActionCategory::Writing, 2.5);
        assert_eq!(a.content(), "write");
        assert_eq!(a.required_time(), 2.5);
        assert!(!a.completed());
    }

    #[test]
    fn test_new_action_with_negative_time_clamps_to_zero() {
        let a = ActionPoint::new("x".into(), ActionCategory::Writing, -3.0);
        assert_eq!(a.required_time(), 0.0);
    }

    #[test]
    fn test_set_required_time_with_negative_clamps_to_zero() {
        let mut a = ActionPoint::new("x".into(), ActionCategory::Writing, 1.0);
        a.set_required_time(-5.0);
        assert_eq!(a.required_time(), 0.0);
    }

    #[test]
    fn test_set_required_time_with_positive_updates_value() {
        let mut a = ActionPoint::new("x".into(), ActionCategory::Writing, 1.0);
        a.set_required_time(4.5);
        assert_eq!(a.required_time(), 4.5);
    }

    #[test]
    fn test_set_completed_when_called_updates_flag() {
        let mut a = ActionPoint::new("x".into(), ActionCategory::Writing, 1.0);
        a.set_completed(true);
        assert!(a.completed());
    }

    #[test]
    fn test_set_category_when_called_updates_category() {
        let mut a = ActionPoint::new("x".into(), ActionCategory::Writing, 1.0);
        a.set_category(ActionCategory::Research);
        assert_eq!(a.category().as_str(), "research");
    }

    #[test]
    fn test_set_content_on_action_when_called_updates_content() {
        let mut a = ActionPoint::new("old".into(), ActionCategory::Writing, 1.0);
        a.set_content("new".into());
        assert_eq!(a.content(), "new");
    }

    // Question

    #[test]
    fn test_new_question_with_priority_initializes_empty() {
        let q = Question::new("hello".into(), QuestionPriority::High);
        assert_eq!(q.content(), "hello");
        assert!(q.actions().is_empty());
        assert!(!q.answered());
    }

    #[test]
    fn test_total_time_required_with_no_actions_returns_zero() {
        let q = Question::new("q".into(), QuestionPriority::Low);
        assert_eq!(q.total_time_required(), 0.0);
    }

    #[test]
    fn test_total_time_required_with_mixed_completion_sums_all() {
        let q = question_with(vec![action(1.0, false), action(2.5, true)], false);
        assert_eq!(q.total_time_required(), 3.5);
    }

    #[test]
    fn test_remaining_time_needed_with_all_completed_returns_zero() {
        let q = question_with(vec![action(1.0, true), action(2.0, true)], false);
        assert_eq!(q.remaining_time_needed(), 0.0);
    }

    #[test]
    fn test_remaining_time_needed_with_mixed_completion_sums_incomplete() {
        let q = question_with(
            vec![action(1.0, false), action(2.5, true), action(0.5, false)],
            false,
        );
        assert_eq!(q.remaining_time_needed(), 1.5);
    }

    #[test]
    fn test_push_action_when_called_appends_to_actions() {
        let mut q = Question::new("q".into(), QuestionPriority::Low);
        q.push_action(action(1.0, false));
        assert_eq!(q.actions().len(), 1);
    }

    #[test]
    fn test_remove_action_with_valid_idx_returns_action_and_shrinks() {
        let mut q = question_with(vec![action(1.0, false), action(2.0, false)], false);
        let removed = q.remove_action(0);
        assert_eq!(removed.required_time(), 1.0);
        assert_eq!(q.actions().len(), 1);
    }

    #[test]
    fn test_action_mut_with_valid_idx_returns_some() {
        let mut q = question_with(vec![action(1.0, false)], false);
        assert!(q.action_mut(0).is_some());
    }

    #[test]
    fn test_action_mut_with_out_of_bounds_returns_none() {
        let mut q = Question::new("q".into(), QuestionPriority::Low);
        assert!(q.action_mut(0).is_none());
    }

    #[test]
    fn test_action_mut_when_mutated_persists_changes() {
        let mut q = question_with(vec![action(1.0, false)], false);
        q.action_mut(0).unwrap().set_completed(true);
        assert!(q.actions()[0].completed());
    }

    #[test]
    fn test_set_priority_when_called_updates_priority() {
        let mut q = Question::new("q".into(), QuestionPriority::Low);
        q.set_priority(QuestionPriority::Critical);
        assert!(matches!(q.priority(), QuestionPriority::Critical));
    }

    #[test]
    fn test_set_answered_with_no_actions_returns_ok_and_updates_flag() {
        let mut q = Question::new("q".into(), QuestionPriority::Low);
        assert!(q.set_answered(true).is_ok());
        assert!(q.answered());
    }

    #[test]
    fn test_set_answered_with_all_complete_returns_ok_and_updates_flag() {
        let mut q = question_with(vec![action(1.0, true), action(2.0, true)], false);
        assert!(q.set_answered(true).is_ok());
        assert!(q.answered());
    }

    #[test]
    fn test_set_answered_with_incomplete_actions_returns_unfinished_error() {
        let mut q = question_with(vec![action(1.0, false), action(2.0, true)], false);
        let remaining_action = q.incomplete_action_count();
        assert_eq!(
            q.set_answered(true),
            Err(QuestionError::IncompleteAction {
                remaining: remaining_action
            })
        );
        assert!(!q.answered());
    }

    #[test]
    fn test_set_answered_to_false_with_incomplete_actions_returns_ok() {
        let mut q = question_with(vec![action(1.0, false)], false);
        assert!(q.set_answered(false).is_ok());
        assert!(!q.answered());
    }

    // Objective

    #[test]
    fn test_new_objective_with_content_initializes_empty_unmet() {
        let o = Objective::new("obj".into());
        assert_eq!(o.content(), "obj");
        assert!(o.questions().is_empty());
        assert!(!o.met());
    }

    #[test]
    fn test_total_allocated_timeframe_with_no_questions_returns_zero() {
        let o = Objective::new("obj".into());
        assert_eq!(o.total_allocated_timeframe(), 0.0);
    }

    #[test]
    fn test_total_allocated_timeframe_with_multiple_questions_sums_all() {
        let mut o = Objective::new("obj".into());
        o.push_question(question_with(
            vec![action(1.0, false), action(2.0, true)],
            false,
        ));
        o.push_question(question_with(vec![action(0.5, true)], true));
        assert_eq!(o.total_allocated_timeframe(), 3.5);
    }

    #[test]
    fn test_remaining_time_needed_with_answered_question_excludes_it() {
        let mut o = Objective::new("obj".into());
        o.push_question(question_with(vec![action(1.0, false)], false));
        o.push_question(question_with(vec![action(5.0, true)], true));
        assert_eq!(o.remaining_time_needed(), 1.0);
    }

    #[test]
    fn test_remaining_time_needed_with_all_unanswered_sums_incomplete_actions() {
        let mut o = Objective::new("obj".into());
        o.push_question(question_with(
            vec![action(1.0, false), action(2.0, true)],
            false,
        ));
        o.push_question(question_with(vec![action(3.0, false)], false));
        assert_eq!(o.remaining_time_needed(), 4.0);
    }

    #[test]
    fn test_push_question_when_called_appends_to_questions() {
        let mut o = Objective::new("obj".into());
        o.push_question(Question::new("q".into(), QuestionPriority::Low));
        assert_eq!(o.questions().len(), 1);
    }

    #[test]
    fn test_remove_question_with_valid_idx_returns_question_and_shrinks() {
        let mut o = Objective::new("obj".into());
        o.push_question(Question::new("first".into(), QuestionPriority::Low));
        o.push_question(Question::new("second".into(), QuestionPriority::High));
        let removed = o.remove_question(0);
        assert_eq!(removed.content(), "first");
        assert_eq!(o.questions().len(), 1);
    }

    #[test]
    fn test_question_mut_with_valid_idx_returns_some() {
        let mut o = Objective::new("obj".into());
        o.push_question(Question::new("q".into(), QuestionPriority::Low));
        assert!(o.question_mut(0).is_some());
    }

    #[test]
    fn test_question_mut_with_out_of_bounds_returns_none() {
        let mut o = Objective::new("obj".into());
        assert!(o.question_mut(0).is_none());
    }

    #[test]
    fn test_question_mut_when_mutated_persists_changes() {
        let mut o = Objective::new("obj".into());
        o.push_question(Question::new("q".into(), QuestionPriority::Low));
        o.question_mut(0).unwrap().set_answered(true).unwrap();
        assert!(o.questions()[0].answered());
    }

    #[test]
    fn test_set_met_with_no_questions_returns_ok_and_updates_flag() {
        let mut o = Objective::new("obj".into());
        assert!(o.set_met(true).is_ok());
        assert!(o.met());
    }

    #[test]
    fn test_set_met_with_all_answered_returns_ok_and_updates_flag() {
        let mut o = Objective::new("obj".into());
        o.push_question(question_with(vec![action(1.0, true)], true));
        o.push_question(question_with(vec![], true));
        assert!(o.set_met(true).is_ok());
        assert!(o.met());
    }

    #[test]
    fn test_set_met_with_unanswered_questions_returns_unanswered_error() {
        let mut o = Objective::new("obj".into());
        o.push_question(question_with(vec![action(1.0, true)], true));
        o.push_question(Question::new("q".into(), QuestionPriority::Low));
        assert_eq!(
            o.set_met(true),
            Err(ObjectiveError::UnansweredQuestion { remaining: 1 })
        );
        assert!(!o.met());
    }

    #[test]
    fn test_set_met_to_false_with_unanswered_questions_returns_ok() {
        let mut o = Objective::new("obj".into());
        o.push_question(Question::new("q".into(), QuestionPriority::Low));
        assert!(o.set_met(false).is_ok());
        assert!(!o.met());
    }

    #[test]
    fn test_set_content_on_objective_when_called_updates_content() {
        let mut o = Objective::new("old".into());
        o.set_content("new".into());
        assert_eq!(o.content(), "new");
    }
}
