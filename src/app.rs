use dioxus::prelude::*;

use crate::components::{GanttView, Nav, TodoView};
use crate::dummy;
use crate::models::{ActionCategory, ActionPoint, Objective, Question, QuestionPriority, Tag};

#[derive(Clone, Copy)]
pub struct AppState {
    pub objectives: Signal<Vec<Objective>>,
    pub last_error: Signal<Option<String>>,
}

impl AppState {
    pub fn new(initial: Vec<Objective>) -> Self {
        Self {
            objectives: Signal::new(initial),
            last_error: Signal::new(None),
        }
    }

    fn report<E: std::fmt::Display>(&mut self, err: E) {
        self.last_error.set(Some(err.to_string()));
    }
    pub fn clear_error(&mut self) {
        self.last_error.set(None);
    }

    pub fn toggle_objective(&mut self, obj: usize) {
        let mut list = self.objectives.write();
        if let Some(o) = list.get_mut(obj) {
            let next = !o.met();
            if let Err(e) = o.set_met(next) {
                drop(list);
                self.report(e);
            }
        }
    }

    pub fn toggle_question(&mut self, obj: usize, q: usize) {
        let mut list = self.objectives.write();
        if let Some(question) = list.get_mut(obj).and_then(|o| o.question_mut(q)) {
            let next = !question.answered();
            if let Err(e) = question.set_answered(next) {
                drop(list);
                self.report(e);
            }
        }
    }

    pub fn toggle_action(&mut self, obj: usize, q: usize, a: usize) {
        let mut list = self.objectives.write();
        if let Some(action) = list
            .get_mut(obj)
            .and_then(|o| o.question_mut(q))
            .and_then(|qq| qq.action_mut(a))
        {
            action.set_completed(!action.completed());
        }
    }

    pub fn add_objective(&mut self, content: String) {
        self.objectives.write().push(Objective::new(content));
    }

    pub fn add_question(&mut self, obj: usize, content: String) {
        if let Some(o) = self.objectives.write().get_mut(obj) {
            o.push_question(Question::new(content, QuestionPriority::Medium));
        }
    }

    pub fn add_action(&mut self, obj: usize, q: usize, content: String) {
        if let Some(question) = self
            .objectives
            .write()
            .get_mut(obj)
            .and_then(|o| o.question_mut(q))
        {
            question.push_action(ActionPoint::new(content, ActionCategory::Writing, 1.0));
        }
    }

    pub fn add_tag(&mut self, obj: usize, name: String) {
        if let Some(o) = self.objectives.write().get_mut(obj) {
            o.add_tag(Tag::new(name));
        }
    }
    pub fn remove_tag(&mut self, obj: usize, name: String) {
        if let Some(o) = self.objectives.write().get_mut(obj) {
            o.remove_tag(&name);
        }
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
    use_context_provider(|| AppState::new(dummy::objectives()));
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
