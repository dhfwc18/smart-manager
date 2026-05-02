use crate::questions::{Objective, ObjectiveError, Question, QuestionError, QuestionPriority, Tag};
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
    DuplicateQuestionId {
        obj_idx: usize,
        id: usize,
    },
    UnknownPrereq {
        obj_idx: usize,
        q_idx: usize,
        prereq_id: usize,
    },
    SelfPrereq {
        obj_idx: usize,
        q_idx: usize,
    },
    PrereqCycle {
        obj_idx: usize,
        q_idx: usize,
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
            Self::DuplicateQuestionId { obj_idx, id } => {
                write!(f, "objective {obj_idx} has duplicate question id {id}")
            }
            Self::UnknownPrereq {
                obj_idx,
                q_idx,
                prereq_id,
            } => write!(
                f,
                "objective {obj_idx} question {q_idx}: prereq id {prereq_id} not found"
            ),
            Self::SelfPrereq { obj_idx, q_idx } => write!(
                f,
                "objective {obj_idx} question {q_idx}: prereq references itself"
            ),
            Self::PrereqCycle { obj_idx, q_idx } => write!(
                f,
                "objective {obj_idx} question {q_idx}: prereq chain forms a cycle"
            ),
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

    pub fn add_question(
        &mut self,
        objective_idx: usize,
        content: String,
        priority: QuestionPriority,
        prereq: Option<usize>,
    ) -> Result<usize, AppError> {
        let objective = self
            .objective_mut(objective_idx)
            .ok_or(AppError::NotFound)?;
        Ok(objective.add_question(content, priority, prereq).id())
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
            validate_tags(oi, o, &mut issues);
            validate_met(oi, o, &mut issues);
            validate_questions(oi, o, &mut issues);
            validate_prereqs(oi, o, &mut issues);
        }
        issues
    }

    pub fn gantt_tasks(&self) -> Vec<GanttTask> {
        objectives_to_gantt_tasks(&self.objectives)
    }

    pub fn render_gantt(&self, format: GanttFormat) -> String {
        gantt::render(&self.objectives, format)
    }

    pub fn save_gantt(&self, format: GanttFormat, path: &Path) -> std::io::Result<()> {
        gantt::save(&self.objectives, format, path)
    }

    pub fn render_todo(&self, format: TodoFormat) -> String {
        todo::render(&self.objectives, format)
    }

    pub fn save_todo(&self, format: TodoFormat, path: &Path) -> std::io::Result<()> {
        todo::save(&self.objectives, format, path)
    }
}

fn validate_tags(oi: usize, o: &Objective, issues: &mut Vec<ValidationIssue>) {
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
}

fn validate_met(oi: usize, o: &Objective, issues: &mut Vec<ValidationIssue>) {
    if !o.met() {
        return;
    }
    let unanswered = o.questions().iter().filter(|q| !q.answered()).count();
    if unanswered > 0 {
        issues.push(ValidationIssue::MetObjectiveWithUnansweredQuestions {
            obj_idx: oi,
            remaining: unanswered,
        });
    }
}

fn validate_questions(oi: usize, o: &Objective, issues: &mut Vec<ValidationIssue>) {
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

fn validate_prereqs(oi: usize, o: &Objective, issues: &mut Vec<ValidationIssue>) {
    let questions = o.questions();
    let mut seen_ids: Vec<usize> = Vec::with_capacity(questions.len());
    for q in questions {
        if seen_ids.contains(&q.id()) {
            issues.push(ValidationIssue::DuplicateQuestionId {
                obj_idx: oi,
                id: q.id(),
            });
        } else {
            seen_ids.push(q.id());
        }
    }

    for (qi, q) in questions.iter().enumerate() {
        let Some(prereq_id) = q.prereq() else {
            continue;
        };
        if prereq_id == q.id() {
            issues.push(ValidationIssue::SelfPrereq {
                obj_idx: oi,
                q_idx: qi,
            });
            continue;
        }
        if o.question_by_id(prereq_id).is_none() {
            issues.push(ValidationIssue::UnknownPrereq {
                obj_idx: oi,
                q_idx: qi,
                prereq_id,
            });
            continue;
        }
        if prereq_chain_returns_to(qi, questions) {
            issues.push(ValidationIssue::PrereqCycle {
                obj_idx: oi,
                q_idx: qi,
            });
        }
    }
}

fn prereq_chain_returns_to(start_idx: usize, questions: &[Question]) -> bool {
    let start_id = questions[start_idx].id();
    let mut cursor_idx = start_idx;
    for _ in 0..questions.len() {
        let Some(prereq_id) = questions[cursor_idx].prereq() else {
            return false;
        };
        let Some(next_idx) = questions.iter().position(|q| q.id() == prereq_id) else {
            return false;
        };
        if questions[next_idx].id() == start_id {
            return true;
        }
        cursor_idx = next_idx;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questions::{ActionCategory, ActionPoint, QuestionPriority};

    fn objective_with_action(content: &str, time: f32, completed: bool) -> Objective {
        let mut o = Objective::new(content.into());
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        let mut a = ActionPoint::new("a".into(), ActionCategory::Writing, time);
        a.set_completed(completed);
        q.push_action(a);
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
    fn test_add_question_with_invalid_idx_returns_not_found() {
        let mut app = App::new();
        assert_eq!(
            app.add_question(0, "q".into(), QuestionPriority::Low, None),
            Err(AppError::NotFound)
        );
    }

    #[test]
    fn test_add_question_with_valid_idx_appends_and_returns_id() {
        let mut app = App::new();
        app.push_objective(Objective::new("o".into()));
        let id = app
            .add_question(0, "q".into(), QuestionPriority::Low, None)
            .unwrap();
        assert_eq!(id, 0);
        assert_eq!(app.objectives()[0].questions().len(), 1);
    }

    #[test]
    fn test_add_question_assigns_increasing_ids() {
        let mut app = App::new();
        app.push_objective(Objective::new("o".into()));
        let id0 = app
            .add_question(0, "a".into(), QuestionPriority::Low, None)
            .unwrap();
        let id1 = app
            .add_question(0, "b".into(), QuestionPriority::Low, Some(id0))
            .unwrap();
        assert_eq!((id0, id1), (0, 1));
        assert_eq!(app.objectives()[0].questions()[1].prereq(), Some(0));
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
        app.add_question(0, "q2".into(), QuestionPriority::High, None)
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
        let json = r#"{"objectives":[{"content":"x","questions":[{"id":0,"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-5.0,"completed":false}],"prereq":null,"answered":false}],"tags":[],"met":false,"next_question_id":1}]}"#;
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
        let json = r#"{"objectives":[{"content":"x","questions":[{"id":0,"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":1.0,"completed":false}],"prereq":null,"answered":true}],"tags":[],"met":false,"next_question_id":1}]}"#;
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
        let json = r#"{"objectives":[{"content":"x","questions":[{"id":0,"content":"q","priority":"Medium","actions":[],"prereq":null,"answered":false}],"tags":[],"met":true,"next_question_id":1}]}"#;
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
        let json = r#"{"objectives":[{"content":"x","questions":[],"tags":["work","work"],"met":false,"next_question_id":0}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            &issues[0],
            ValidationIssue::DuplicateTag { obj_idx: 0, name } if name == "work"
        ));
    }

    #[test]
    fn test_validate_with_duplicate_question_id_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":1,"content":"a","priority":"Medium","actions":[],"prereq":null,"answered":false},
            {"id":1,"content":"b","priority":"Medium","actions":[],"prereq":null,"answered":false}
        ],"tags":[],"met":false,"next_question_id":2}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert!(matches!(
            issues[0],
            ValidationIssue::DuplicateQuestionId { obj_idx: 0, id: 1 }
        ));
    }

    #[test]
    fn test_validate_with_unknown_prereq_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":0,"content":"a","priority":"Medium","actions":[],"prereq":99,"answered":false}
        ],"tags":[],"met":false,"next_question_id":1}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert!(matches!(
            issues[0],
            ValidationIssue::UnknownPrereq {
                obj_idx: 0,
                q_idx: 0,
                prereq_id: 99
            }
        ));
    }

    #[test]
    fn test_validate_with_self_prereq_reports_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":7,"content":"a","priority":"Medium","actions":[],"prereq":7,"answered":false}
        ],"tags":[],"met":false,"next_question_id":8}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        assert!(matches!(
            issues[0],
            ValidationIssue::SelfPrereq {
                obj_idx: 0,
                q_idx: 0
            }
        ));
    }

    #[test]
    fn test_validate_with_prereq_cycle_reports_issue() {
        // a -> b -> a
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":0,"content":"a","priority":"Medium","actions":[],"prereq":1,"answered":false},
            {"id":1,"content":"b","priority":"Medium","actions":[],"prereq":0,"answered":false}
        ],"tags":[],"met":false,"next_question_id":2}]}"#;
        let app = App::from_json(json).unwrap();
        let issues = app.validate();
        let cycles: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i, ValidationIssue::PrereqCycle { .. }))
            .collect();
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn test_validate_allows_answered_question_with_unanswered_prereq() {
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":0,"content":"a","priority":"Medium","actions":[],"prereq":null,"answered":false},
            {"id":1,"content":"b","priority":"Medium","actions":[],"prereq":0,"answered":true}
        ],"tags":[],"met":false,"next_question_id":2}]}"#;
        let app = App::from_json(json).unwrap();
        assert!(app.validate().is_empty());
    }

    #[test]
    fn test_validate_with_valid_prereq_chain_reports_no_issue() {
        let json = r#"{"objectives":[{"content":"x","questions":[
            {"id":0,"content":"a","priority":"Medium","actions":[],"prereq":null,"answered":false},
            {"id":1,"content":"b","priority":"Medium","actions":[],"prereq":0,"answered":false},
            {"id":2,"content":"c","priority":"Medium","actions":[],"prereq":1,"answered":false}
        ],"tags":[],"met":false,"next_question_id":3}]}"#;
        let app = App::from_json(json).unwrap();
        assert!(app.validate().is_empty());
    }

    #[test]
    fn test_validate_collects_all_issues() {
        let json = r#"{"objectives":[
            {"content":"a","questions":[{"id":0,"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-1.0,"completed":false}],"prereq":null,"answered":true}],"tags":["x","x"],"met":false,"next_question_id":1},
            {"content":"b","questions":[{"id":0,"content":"q","priority":"Medium","actions":[],"prereq":null,"answered":false}],"tags":[],"met":true,"next_question_id":1}
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
            r#"{"objectives":[{"content":"x","questions":[{"id":0,"content":"q","priority":"Medium","actions":[{"content":"a","category":"Writing","required_time":-5.0,"completed":false}],"prereq":null,"answered":false}],"tags":[],"met":false,"next_question_id":1}]}"#,
        )
        .unwrap();
        let err = App::load(&path).err().expect("load should fail");
        assert!(matches!(err, PersistError::Validation(_)));
        let _ = std::fs::remove_file(&path);
    }
}
