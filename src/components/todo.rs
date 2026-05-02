use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::{Objective, Question};

#[component]
pub fn TodoView() -> Element {
    let state = use_context::<AppState>();
    let objectives = state.objectives;
    let snapshot = objectives.read().clone();

    rsx! {
        section { class: "todo-view",
            div { class: "view-header",
                h1 { "Objectives" }
                AddObjectiveInput {}
            }
            if snapshot.is_empty() {
                div { class: "empty", "No objectives yet — add one above." }
            }
            div { class: "objective-list",
                for (i, obj) in snapshot.into_iter().enumerate() {
                    ObjectiveBlock { key: "{i}", index: i, obj }
                }
            }
        }
    }
}

#[component]
fn ObjectiveBlock(index: usize, obj: Objective) -> Element {
    let mut state = use_context::<AppState>();
    let met = obj.met();
    let questions = obj.questions().to_vec();
    let tags: Vec<String> = obj.tags().iter().map(|t| t.name().to_string()).collect();

    rsx! {
        article { class: if met { "objective done" } else { "objective" },
            header { class: "objective-header",
                input {
                    r#type: "checkbox",
                    checked: met,
                    onchange: move |_| state.toggle_objective(index),
                }
                span { class: "objective-title", "{obj.content()}" }
                div { class: "tag-list",
                    for tag in tags.iter().cloned() {
                        TagChip { obj_index: index, name: tag }
                    }
                    AddTagInput { obj_index: index }
                }
            }
            div { class: "question-list",
                for (qi, question) in questions.into_iter().enumerate() {
                    QuestionBlock { key: "{qi}", obj_index: index, q_index: qi, question }
                }
                AddQuestionInput { obj_index: index }
            }
        }
    }
}

#[component]
fn QuestionBlock(obj_index: usize, q_index: usize, question: Question) -> Element {
    let mut state = use_context::<AppState>();
    let answered = question.answered();
    let priority = question.priority().as_str().to_string();
    let actions = question.actions().to_vec();

    rsx! {
        div { class: if answered { "question done" } else { "question" },
            div { class: "question-header",
                input {
                    r#type: "checkbox",
                    checked: answered,
                    onchange: move |_| state.toggle_question(obj_index, q_index),
                }
                span { class: "question-title", "{question.content()}" }
                span { class: "priority priority-{priority}", "{priority}" }
            }
            div { class: "action-list",
                for (ai, action) in actions.into_iter().enumerate() {
                    ActionRow {
                        key: "{ai}",
                        obj_index,
                        q_index,
                        a_index: ai,
                        title: action.content().to_string(),
                        category: action.category().as_str().to_string(),
                        required_time: action.required_time(),
                        completed: action.completed(),
                    }
                }
                AddActionInput { obj_index, q_index }
            }
        }
    }
}

#[component]
fn ActionRow(
    obj_index: usize,
    q_index: usize,
    a_index: usize,
    title: String,
    category: String,
    required_time: f32,
    completed: bool,
) -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: if completed { "action-row done" } else { "action-row" },
            input {
                r#type: "checkbox",
                checked: completed,
                onchange: move |_| state.toggle_action(obj_index, q_index, a_index),
            }
            span { class: "action-title", "{title}" }
            span { class: "category", "{category}" }
            span { class: "time", "{required_time}h" }
        }
    }
}

#[component]
fn AddObjectiveInput() -> Element {
    let mut state = use_context::<AppState>();
    let mut draft = use_signal(String::new);

    let mut commit = move || {
        let value = draft.read().trim().to_string();
        if !value.is_empty() {
            state.add_objective(value);
            draft.write().clear();
        }
    };

    rsx! {
        div { class: "inline-add",
            input {
                r#type: "text",
                placeholder: "New objective…",
                value: "{draft}",
                oninput: move |e| *draft.write() = e.value(),
                onkeydown: move |e| if e.key() == Key::Enter { commit() },
            }
            button { onclick: move |_| commit(), "Add" }
        }
    }
}

#[component]
fn AddQuestionInput(obj_index: usize) -> Element {
    let mut state = use_context::<AppState>();
    let mut draft = use_signal(String::new);

    let mut commit = move || {
        let value = draft.read().trim().to_string();
        if !value.is_empty() {
            state.add_question(obj_index, value);
            draft.write().clear();
        }
    };

    rsx! {
        div { class: "inline-add nested",
            input {
                r#type: "text",
                placeholder: "New question…",
                value: "{draft}",
                oninput: move |e| *draft.write() = e.value(),
                onkeydown: move |e| if e.key() == Key::Enter { commit() },
            }
            button { onclick: move |_| commit(), "Add" }
        }
    }
}

#[component]
fn AddActionInput(obj_index: usize, q_index: usize) -> Element {
    let mut state = use_context::<AppState>();
    let mut draft = use_signal(String::new);

    let mut commit = move || {
        let value = draft.read().trim().to_string();
        if !value.is_empty() {
            state.add_action(obj_index, q_index, value);
            draft.write().clear();
        }
    };

    rsx! {
        div { class: "inline-add nested-deep",
            input {
                r#type: "text",
                placeholder: "New action…",
                value: "{draft}",
                oninput: move |e| *draft.write() = e.value(),
                onkeydown: move |e| if e.key() == Key::Enter { commit() },
            }
            button { onclick: move |_| commit(), "Add" }
        }
    }
}

#[component]
fn TagChip(obj_index: usize, name: String) -> Element {
    let mut state = use_context::<AppState>();
    let label = name.clone();
    rsx! {
        span { class: "tag-chip",
            "{label}"
            button {
                class: "tag-remove",
                onclick: move |_| state.remove_tag(obj_index, name.clone()),
                "✕"
            }
        }
    }
}

#[component]
fn AddTagInput(obj_index: usize) -> Element {
    let mut state = use_context::<AppState>();
    let mut draft = use_signal(String::new);

    let mut commit = move || {
        let value = draft.read().trim().to_string();
        if !value.is_empty() {
            state.add_tag(obj_index, value);
            draft.write().clear();
        }
    };

    rsx! {
        span { class: "tag-add",
            input {
                r#type: "text",
                placeholder: "+tag",
                value: "{draft}",
                oninput: move |e| *draft.write() = e.value(),
                onkeydown: move |e| if e.key() == Key::Enter { commit() },
            }
        }
    }
}
