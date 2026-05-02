use crate::questions::{Objective, ObjectiveError, Question, QuestionError, Tag};
use crate::writer::gantt::{self, GanttFormat, GanttTask};
use crate::writer::objectives_to_gantt_tasks;
use crate::writer::todo::{self, TodoFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

#[derive(Debug)]
pub enum PersistError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    Validation(Vec<ValidationIssue>),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Serde(e) => write!(f, "serde: {e}"),
            Self::Validation(issues) => {
                writeln!(f, "validation failed ({} issue(s)):", issues.len())?;
                for issue in issues {
                    writeln!(f, "  - {issue}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PersistError {}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationIssue {
    NegativeRequiredTime {
        obj_idx: usize,
        q_idx: usize,
        action_idx: usize,
    },
    AnsweredQuestionWithIncompleteActions {
        obj_idx: usize,
        q_idx: usize,
        remaining: usize,
    },
    MetObjectiveWithUnansweredQuestions {
        obj_idx: usize,
        remaining: usize,
    },
    DuplicateTag {
        obj_idx: usize,
        name: String,
    },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeRequiredTime {
                obj_idx,
                q_idx,
                action_idx,
            } => write!(
                f,
                "objective {obj_idx} question {q_idx} action {action_idx}: required_time is negative"
            ),
            Self::AnsweredQuestionWithIncompleteActions {
                obj_idx,
                q_idx,
                remaining,
            } => write!(
                f,
                "objective {obj_idx} question {q_idx} answered but {remaining} action(s) incomplete"
            ),
            Self::MetObjectiveWithUnansweredQuestions { obj_idx, remaining } => write!(
                f,
                "objective {obj_idx} met but {remaining} question(s) unanswered"
            ),
            Self::DuplicateTag { obj_idx, name } => {
                write!(f, "objective {obj_idx} has duplicate tag {name:?}")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
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

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn save(&self, path: &Path) -> Result<(), PersistError> {
        let json = self.to_json()?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, PersistError> {
        let data = fs::read_to_string(path)?;
        let app = Self::from_json(&data)?;
        let issues = app.validate();
        if !issues.is_empty() {
            return Err(PersistError::Validation(issues));
        }
        Ok(app)
    }

    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        for (oi, o) in self.objectives.iter().enumerate() {
            let mut seen: Vec<&str> = Vec::new();
            for t in o.tags() {
                if seen.contains(&t.name()) {
                    issues.push(ValidationIssue::DuplicateTag {
                        obj_idx: oi,
                        name: t.name().to_string(),
                    });
                } else {
                    seen.push(t.name());
                }
            }
            if o.met() {
                let unanswered = o.questions().iter().filter(|q| !q.answered()).count();
                if unanswered > 0 {
                    issues.push(ValidationIssue::MetObjectiveWithUnansweredQuestions {
                        obj_idx: oi,
                        remaining: unanswered,
                    });
                }
            }
            for (qi, q) in o.questions().iter().enumerate() {
                if q.answered() {
                    let incomplete = q.actions().iter().filter(|a| !a.completed()).count();
                    if incomplete > 0 {
                        issues.push(ValidationIssue::AnsweredQuestionWithIncompleteActions {
                            obj_idx: oi,
                            q_idx: qi,
                            remaining: incomplete,
                        });
                    }
                }
                for (ai, a) in q.actions().iter().enumerate() {
                    if a.required_time() < 0.0 {
                        issues.push(ValidationIssue::NegativeRequiredTime {
                            obj_idx: oi,
                            q_idx: qi,
                            action_idx: ai,
                        });
                    }
                }
            }
        }
        issues
    }

    pub fn gantt_tasks(&self) -> Vec<GanttTask> {
        objectives_to_gantt_tasks(&self.objectives)
    }

    pub fn render_gantt(&self, format: GanttFormat) -> String {
        gantt::render(&self.gantt_tasks(), format)
    }

    pub fn save_gantt(&self, format: GanttFormat, path: &Path) -> std::io::Result<()> {
        gantt::save(&self.gantt_tasks(), format, path)
    }

    pub fn render_todo(&self, format: TodoFormat) -> String {
        todo::render(&self.gantt_tasks(), format)
    }

    pub fn save_todo(&self, format: TodoFormat, path: &Path) -> std::io::Result<()> {
        todo::save(&self.gantt_tasks(), format, path)
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

    fn populated_app() -> App {
        let mut app = App::new();
        app.push_objective(objective_with_action("a", 1.5, false));
        app.push_objective(objective_with_action("b", 2.0, true));
        app.add_tag(0, Tag::new("work")).unwrap();
        app.add_tag(1, Tag::new("home")).unwrap();
        app.push_question(0, Question::new("q2".into(), QuestionPriority::High))
            .unwrap();
        app
    }

    #[test]
    fn test_to_json_emits_object_with_objectives_array() {
        let app = populated_app();
        let json = app.to_json().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["objectives"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_from_json_round_trips_through_to_json() {
        let app = populated_app();
        let json = app.to_json().unwrap();
        let restored = App::from_json(&json).unwrap();
        assert_eq!(restored.objectives().len(), app.objectives().len());
        assert_eq!(restored.objectives()[0].content(), "a");
        assert!(restored.objectives()[0].has_tag("work"));
        assert!(restored.objectives()[1].has_tag("home"));
        assert_eq!(
            restored.remaining_time_needed(),
            app.remaining_time_needed()
        );
        assert_eq!(
            restored.objectives()[0].questions().len(),
            app.objectives()[0].questions().len()
        );
    }

    #[test]
    fn test_save_and_load_persist_app_state() {
        let path = std::env::temp_dir().join("smart_manager_app_test.json");
        let _ = std::fs::remove_file(&path);
        let app = populated_app();
        app.save(&path).unwrap();
        let restored = App::load(&path).unwrap();
        assert_eq!(restored.objectives().len(), app.objectives().len());
        assert_eq!(
            restored.objectives()[0].questions().len(),
            app.objectives()[0].questions().len()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_with_missing_file_returns_io_error() {
        let path = std::env::temp_dir().join("smart_manager_app_definitely_missing.json");
        let _ = std::fs::remove_file(&path);
        let err = App::load(&path).err().expect("load should fail");
        assert!(matches!(err, PersistError::Io(_)));
    }

    #[test]
    fn test_from_json_with_invalid_json_returns_error() {
        assert!(App::from_json("{ not json").is_err());
    }

    #[test]
    fn test_render_gantt_ascii_includes_objective_content() {
        let mut app = App::new();
        app.push_objective(objective_with_action("Foo", 2.0, false));
        let out = app.render_gantt(GanttFormat::Ascii);
        assert!(out.contains("Foo"));
    }

    #[test]
    fn test_render_todo_markdown_includes_checkbox_for_objective() {
        let mut app = App::new();
        app.push_objective(objective_with_action("Foo", 1.0, false));
        let out = app.render_todo(TodoFormat::Markdown);
        assert!(out.contains("- [ ]"));
        assert!(out.contains("Foo"));
    }

    #[test]
    fn test_save_gantt_writes_file_with_rendered_contents() {
        let path = std::env::temp_dir().join("smart_manager_app_gantt_test.md");
        let _ = std::fs::remove_file(&path);
        let mut app = App::new();
        app.push_objective(objective_with_action("Foo", 1.0, false));
        app.save_gantt(GanttFormat::Markdown, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("Foo"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_validate_with_clean_app_returns_empty_vec() {
        let app = populated_app();
        assert!(app.validate().is_empty());
    }

    #[test]
    fn test_validate_with_negative_required_time_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[{"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-5.0,"completed":false}],"answered":false}],"tags":[],"met":false}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            issues[0],
            ValidationIssue::NegativeRequiredTime {
                obj_idx: 0,
                q_idx: 0,
                action_idx: 0
            }
        ));
    }

    #[test]
    fn test_validate_with_answered_question_having_incomplete_actions_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[{"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":1.0,"completed":false}],"answered":true}],"tags":[],"met":false}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert!(matches!(
            issues[0],
            ValidationIssue::AnsweredQuestionWithIncompleteActions {
                obj_idx: 0,
                q_idx: 0,
                remaining: 1
            }
        ));
    }

    #[test]
    fn test_validate_with_met_objective_having_unanswered_questions_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[{"content":"q","priority":"Medium","actions":[],"answered":false}],"tags":[],"met":true}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert!(matches!(
            issues[0],
            ValidationIssue::MetObjectiveWithUnansweredQuestions {
                obj_idx: 0,
                remaining: 1
            }
        ));
    }

    #[test]
    fn test_validate_with_duplicate_tags_reports_issue() {
        let json =
            r#"{"objectives":[{"content":"x","questions":[],"tags":["work","work"],"met":false}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            &issues[0],
            ValidationIssue::DuplicateTag { obj_idx: 0, name } if name == "work"
        ));
    }

    #[test]
    fn test_validate_collects_all_issues() {
        let json = r#"{"objectives":[
            {"content":"a","questions":[{"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-1.0,"completed":false}],"answered":true}],"tags":["x","x"],"met":false},
            {"content":"b","questions":[{"content":"q","priority":"Medium","actions":[],"answered":false}],"tags":[],"met":true}
        ]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert_eq!(issues.len(), 4);
    }

    #[test]
    fn test_load_with_invariant_violation_returns_validation_error() {
        let path = std::env::temp_dir().join("smart_manager_app_invalid.json");
        std::fs::write(
            &path,
            r#"{"objectives":[{"content":"x","questions":[{"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-5.0,"completed":false}],"answered":false}],"tags":[],"met":false}]}"#,
        )
        .unwrap();
        let err = App::load(&path).err().expect("load should fail");
        assert!(matches!(err, PersistError::Validation(_)));
        let _ = std::fs::remove_file(&path);
    }
}
