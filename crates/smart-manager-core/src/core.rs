use crate::questions::{Objective, ObjectiveError, Question, QuestionError, Tag};

#[derive(Debug, PartialEq, Eq)]
pub enum AppError {
    NotFound,
    Question(QuestionError),
    Objective(ObjectiveError),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "objective or question not found"),
            Self::Question(e) => write!(f, "{e}"),
            Self::Objective(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AppError {}

pub struct App {
    objectives: Vec<Objective>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
        }
    }

    pub fn objectives(&self) -> &[Objective] {
        &self.objectives
    }

    pub fn objective_mut(&mut self, idx: usize) -> Option<&mut Objective> {
        self.objectives.get_mut(idx)
    }

    pub fn push_objective(&mut self, objective: Objective) {
        self.objectives.push(objective);
    }
    pub fn remove_objective(&mut self, idx: usize) -> Objective {
        self.objectives.remove(idx)
    }

    pub fn push_question(
        &mut self,
        objective_idx: usize,
        question: Question,
    ) -> Result<(), AppError> {
        self.objective_mut(objective_idx)
            .ok_or(AppError::NotFound)?
            .push_question(question);
        Ok(())
    }

    pub fn add_tag(&mut self, objective_idx: usize, tag: Tag) -> Result<bool, AppError> {
        Ok(self
            .objective_mut(objective_idx)
            .ok_or(AppError::NotFound)?
            .add_tag(tag))
    }
    pub fn remove_tag(
        &mut self,
        objective_idx: usize,
        name: &str,
    ) -> Result<Option<Tag>, AppError> {
        Ok(self
            .objective_mut(objective_idx)
            .ok_or(AppError::NotFound)?
            .remove_tag(name))
    }

    pub fn objectives_with_tag(&self, name: &str) -> Vec<&Objective> {
        self.objectives.iter().filter(|o| o.has_tag(name)).collect()
    }

    pub fn tags(&self) -> Vec<Tag> {
        let mut seen: Vec<Tag> = Vec::new();
        for o in &self.objectives {
            for t in o.tags() {
                if !seen.iter().any(|s| s.name() == t.name()) {
                    seen.push(t.clone());
                }
            }
        }
        seen
    }

    pub fn total_allocated_timeframe(&self) -> f32 {
        self.objectives
            .iter()
            .map(|o| o.total_allocated_timeframe())
            .sum()
    }
    pub fn remaining_time_needed(&self) -> f32 {
        self.objectives
            .iter()
            .filter(|o| !o.met())
            .map(|o| o.remaining_time_needed())
            .sum()
    }

    pub fn remaining_time_needed_for_tag(&self, name: &str) -> f32 {
        self.objectives_with_tag(name)
            .iter()
            .filter(|o| !o.met())
            .map(|o| o.remaining_time_needed())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questions::{ActionCategory, ActionPoint, QuestionPriority};

    fn objective_with_action(content: &str, time: f32, completed: bool) -> Objective {
        let mut o = Objective::new(content.into());
        let mut q = Question::new("q".into(), QuestionPriority::Medium);
        let mut a = ActionPoint::new("a".into(), ActionCategory::Writing, time);
        a.set_completed(completed);
        q.push_action(a);
        o.push_question(q);
        o
    }

    #[test]
    fn test_new_app_has_no_objectives() {
        let app = App::new();
        assert!(app.objectives().is_empty());
    }

    #[test]
    fn test_push_objective_when_called_appends() {
        let mut app = App::new();
        app.push_objective(Objective::new("o".into()));
        assert_eq!(app.objectives().len(), 1);
    }

    #[test]
    fn test_push_question_with_invalid_idx_returns_not_found() {
        let mut app = App::new();
        assert_eq!(
            app.push_question(0, Question::new("q".into(), QuestionPriority::Low)),
            Err(AppError::NotFound)
        );
    }

    #[test]
    fn test_push_question_with_valid_idx_appends() {
        let mut app = App::new();
        app.push_objective(Objective::new("o".into()));
        app.push_question(0, Question::new("q".into(), QuestionPriority::Low))
            .unwrap();
        assert_eq!(app.objectives()[0].questions().len(), 1);
    }

    #[test]
    fn test_add_tag_with_new_tag_returns_true() {
        let mut app = App::new();
        app.push_objective(Objective::new("o".into()));
        assert_eq!(app.add_tag(0, Tag::new("work")), Ok(true));
    }

    #[test]
    fn test_add_tag_with_invalid_idx_returns_not_found() {
        let mut app = App::new();
        assert_eq!(app.add_tag(0, Tag::new("work")), Err(AppError::NotFound));
    }

    #[test]
    fn test_objectives_with_tag_returns_only_tagged() {
        let mut app = App::new();
        app.push_objective(Objective::new("a".into()));
        app.push_objective(Objective::new("b".into()));
        app.push_objective(Objective::new("c".into()));
        app.add_tag(0, Tag::new("work")).unwrap();
        app.add_tag(2, Tag::new("work")).unwrap();
        let tagged = app.objectives_with_tag("work");
        assert_eq!(tagged.len(), 2);
        assert_eq!(tagged[0].content(), "a");
        assert_eq!(tagged[1].content(), "c");
    }

    #[test]
    fn test_tags_aggregates_unique_tags_across_objectives() {
        let mut app = App::new();
        app.push_objective(Objective::new("a".into()));
        app.push_objective(Objective::new("b".into()));
        app.add_tag(0, Tag::new("work")).unwrap();
        app.add_tag(1, Tag::new("work")).unwrap();
        app.add_tag(1, Tag::new("personal")).unwrap();
        let mut tags: Vec<String> = app.tags().iter().map(|t| t.name().to_string()).collect();
        tags.sort();
        assert_eq!(tags, vec!["personal", "work"]);
    }

    #[test]
    fn test_remaining_time_needed_aggregates_across_objectives() {
        let mut app = App::new();
        app.push_objective(objective_with_action("a", 1.5, false));
        app.push_objective(objective_with_action("b", 2.0, true));
        assert_eq!(app.remaining_time_needed(), 1.5);
    }

    #[test]
    fn test_remaining_time_needed_for_tag_filters_by_tag() {
        let mut app = App::new();
        app.push_objective(objective_with_action("a", 1.5, false));
        app.push_objective(objective_with_action("b", 4.0, false));
        app.add_tag(0, Tag::new("work")).unwrap();
        assert_eq!(app.remaining_time_needed_for_tag("work"), 1.5);
    }
}
