use dioxus::prelude::*;
use smart_manager_core::core::App;
use smart_manager_core::questions::{
    ActionCategory, ActionPoint, Objective, Question, QuestionPriority, Tag,
};

use crate::components::{GanttView, Nav, TodoView};

#[derive(Clone, Copy)]
pub struct AppState {
    app: Signal<App>,
    pub view: Signal<Vec<ObjectiveView>>,
    pub last_error: Signal<Option<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveView {
    pub content: String,
    pub tags: Vec<String>,
    pub met: bool,
    pub questions: Vec<QuestionView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionView {
    pub content: String,
    pub priority: String,
    pub answered: bool,
    pub actions: Vec<ActionView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionView {
    pub content: String,
    pub category: String,
    pub required_time: f32,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GanttBarData {
    pub name: String,
    pub days: f32,
    pub priority: String,
    pub group: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let app = App::new();
        let view = Signal::new(snapshot(&app));
        Self {
            app: Signal::new(app),
            view,
            last_error: Signal::new(None),
        }
    }

    fn refresh(&mut self) {
        let v = snapshot(&self.app.read());
        self.view.set(v);
    }

    fn report(&mut self, message: String) {
        self.last_error.set(Some(message));
    }
    pub fn clear_error(&mut self) {
        self.last_error.set(None);
    }

    pub fn toggle_objective(&mut self, obj: usize) -> Result<(), String> {
        let res = {
            let mut app = self.app.write();
            match app.objective_mut(obj) {
                Some(o) => {
                    let next = !o.met();
                    o.set_met(next).map_err(|e| e.to_string())
                }
                None => Err("objective not found".into()),
            }
        };
        match res {
            Ok(()) => {
                self.refresh();
                Ok(())
            }
            Err(e) => {
                self.report(e.clone());
                Err(e)
            }
        }
    }

    pub fn toggle_question(&mut self, obj: usize, q: usize) -> Result<(), String> {
        let res = {
            let mut app = self.app.write();
            match app.objective_mut(obj).and_then(|o| o.question_mut(q)) {
                Some(question) => {
                    let next = !question.answered();
                    question.set_answered(next).map_err(|e| e.to_string())
                }
                None => Err("question not found".into()),
            }
        };
        match res {
            Ok(()) => {
                self.refresh();
                Ok(())
            }
            Err(e) => {
                self.report(e.clone());
                Err(e)
            }
        }
    }

    pub fn toggle_action(&mut self, obj: usize, q: usize, a: usize) {
        {
            let mut app = self.app.write();
            if let Some(action) = app
                .objective_mut(obj)
                .and_then(|o| o.question_mut(q))
                .and_then(|qq| qq.action_mut(a))
            {
                action.set_completed(!action.completed());
            }
        }
        self.refresh();
    }

    pub fn add_objective(&mut self, content: String) {
        self.app.write().push_objective(Objective::new(content));
        self.refresh();
    }

    pub fn add_question(&mut self, obj: usize, content: String) {
        let _ = self
            .app
            .write()
            .push_question(obj, Question::new(content, QuestionPriority::Medium));
        self.refresh();
    }

    pub fn add_action(
        &mut self,
        obj: usize,
        q: usize,
        content: String,
        category: String,
        required_time: f32,
    ) {
        let category = parse_category(&category);
        {
            let mut app = self.app.write();
            if let Some(question) = app.objective_mut(obj).and_then(|o| o.question_mut(q)) {
                question.push_action(ActionPoint::new(content, category, required_time));
            }
        }
        self.refresh();
    }

    pub fn add_tag(&mut self, obj: usize, name: String) {
        let _ = self.app.write().add_tag(obj, Tag::new(name));
        self.refresh();
    }

    pub fn remove_tag(&mut self, obj: usize, name: String) {
        let _ = self.app.write().remove_tag(obj, &name);
        self.refresh();
    }

    pub fn gantt_bars(&self) -> Vec<GanttBarData> {
        self.app
            .read()
            .gantt_tasks()
            .into_iter()
            .map(|t| GanttBarData {
                name: t.name,
                days: t.days,
                priority: t.priority.label().to_string(),
                group: t.group,
            })
            .collect()
    }
}

fn snapshot(app: &App) -> Vec<ObjectiveView> {
    app.objectives()
        .iter()
        .map(|o| ObjectiveView {
            content: o.content().to_string(),
            tags: o.tags().iter().map(|t| t.name().to_string()).collect(),
            met: o.met(),
            questions: o
                .questions()
                .iter()
                .map(|q| QuestionView {
                    content: q.content().to_string(),
                    priority: priority_str(q.priority()).to_string(),
                    answered: q.answered(),
                    actions: q
                        .actions()
                        .iter()
                        .map(|a| ActionView {
                            content: a.content().to_string(),
                            category: a.category().as_str().to_string(),
                            required_time: a.required_time(),
                            completed: a.completed(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

pub const ACTION_CATEGORIES: &[&str] = &[
    "writing",
    "managing",
    "qa",
    "analysis",
    "research",
    "programming",
    "presentation",
];

fn parse_category(s: &str) -> ActionCategory {
    match s {
        "managing" => ActionCategory::Managing,
        "qa" => ActionCategory::Qa,
        "analysis" => ActionCategory::Analysis,
        "research" => ActionCategory::Research,
        "programming" => ActionCategory::Programming,
        "presentation" => ActionCategory::Presentation,
        _ => ActionCategory::Writing,
    }
}

fn priority_str(p: &QuestionPriority) -> &'static str {
    match p {
        QuestionPriority::Critical => "critical",
        QuestionPriority::High => "high",
        QuestionPriority::Medium => "medium",
        QuestionPriority::Low => "low",
        QuestionPriority::LongTerm => "long-term",
    }
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Shell)]
    #[route("/")]
    TodoView {},
    #[route("/gantt")]
    GanttView {},
}

#[component]
pub fn Shell() -> Element {
    use_context_provider(|| AppState::new());
    let mut state = use_context::<AppState>();
    let err = state.last_error;

    rsx! {
        Nav {}
        if let Some(message) = err.read().clone() {
            div { class: "error-banner",
                span { "{message}" }
                button { onclick: move |_| state.clear_error(), "Dismiss" }
            }
        }
        main { class: "main-content", Outlet::<Route> {} }
    }
}
