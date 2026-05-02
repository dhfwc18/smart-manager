use dioxus::prelude::*;

use crate::app::{ACTION_CATEGORIES, ActionView, AppState, ObjectiveView, QuestionView};

#[component]
pub fn TodoView() -> Element {
    let state = use_context::<AppState>();
    let snapshot = state.view.read().clone();

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
fn ObjectiveBlock(index: usize, obj: ObjectiveView) -> Element {
    let mut state = use_context::<AppState>();
    let met = obj.met;
    let tags = obj.tags.clone();
    let questions = obj.questions.clone();

    rsx! {
        article { class: if met { "objective done" } else { "objective" },
            header { class: "objective-header",
                input {
                    r#type: "checkbox",
                    checked: met,
                    onclick: move |e| {
                        if state.toggle_objective(index).is_err() {
                            e.prevent_default();
                        }
                    },
                }
                span { class: "objective-title", "{obj.content}" }
                div { class: "tag-list",
                    for tag in tags.iter().cloned() {
                        TagChip { obj_index: index, name: tag }
                    }
                    AddTagInput { obj_index: index }
                }
            }
            div { class: "question-list",
                for (qi, q) in questions.into_iter().enumerate() {
                    QuestionBlock { key: "{qi}", obj_index: index, q_index: qi, q }
                }
                AddQuestionInput { obj_index: index }
            }
        }
    }
}

#[component]
fn QuestionBlock(obj_index: usize, q_index: usize, q: QuestionView) -> Element {
    let mut state = use_context::<AppState>();
    let answered = q.answered;
    let priority = q.priority.clone();
    let actions = q.actions.clone();

    rsx! {
        div { class: if answered { "question done" } else { "question" },
            div { class: "question-header",
                input {
                    r#type: "checkbox",
                    checked: answered,
                    onclick: move |e| {
                        if state.toggle_question(obj_index, q_index).is_err() {
                            e.prevent_default();
                        }
                    },
                }
                span { class: "question-title", "{q.content}" }
                span { class: "priority priority-{priority}", "{priority}" }
            }
            div { class: "action-list",
                for (ai, a) in actions.into_iter().enumerate() {
                    ActionRow {
                        key: "{ai}",
                        obj_index,
                        q_index,
                        a_index: ai,
                        action: a,
                    }
                }
                AddActionInput { obj_index, q_index }
            }
        }
    }
}

#[component]
fn ActionRow(obj_index: usize, q_index: usize, a_index: usize, action: ActionView) -> Element {
    let mut state = use_context::<AppState>();
    let completed = action.completed;
    rsx! {
        div { class: if completed { "action-row done" } else { "action-row" },
            input {
                r#type: "checkbox",
                checked: completed,
                onchange: move |_| state.toggle_action(obj_index, q_index, a_index),
            }
            span { class: "action-title", "{action.content}" }
            span { class: "category", "{action.category}" }
            span { class: "time", "{action.required_time}d" }
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
    let mut category = use_signal(|| ACTION_CATEGORIES[0].to_string());
    let mut time_raw = use_signal(|| "1".to_string());

    let mut commit = move || {
        let value = draft.read().trim().to_string();
        if !value.is_empty() {
            let time = time_raw
                .read()
                .trim()
                .parse::<f32>()
                .unwrap_or(0.0)
                .max(0.0);
            state.add_action(obj_index, q_index, value, category.read().clone(), time);
            draft.write().clear();
            time_raw.set("1".to_string());
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
            select {
                class: "category-select",
                value: "{category}",
                onchange: move |e| category.set(e.value()),
                for opt in ACTION_CATEGORIES.iter() {
                    option { value: "{opt}", "{opt}" }
                }
            }
            input {
                class: "time-input",
                r#type: "number",
                step: "0.25",
                min: "0",
                value: "{time_raw}",
                oninput: move |e| time_raw.set(e.value()),
                onkeydown: move |e| if e.key() == Key::Enter { commit() },
            }
            span { class: "time-suffix", "d" }
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
