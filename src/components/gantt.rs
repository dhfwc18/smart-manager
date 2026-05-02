use dioxus::prelude::*;

use crate::app::AppState;

const PX_PER_HOUR: f64 = 24.0;
const ROW_HEIGHT: f64 = 32.0;

struct Bar {
    objective: String,
    question: String,
    action: String,
    category: String,
    start_hours: f64,
    duration_hours: f64,
    completed: bool,
}

#[component]
pub fn GanttView() -> Element {
    let state = use_context::<AppState>();
    let snapshot = state.objectives.read().clone();

    // Naive layout: stack actions sequentially within each question.
    // The real engine will replace start_hours with scheduled times.
    let mut bars: Vec<Bar> = Vec::new();
    let mut max_end: f64 = 0.0;
    for obj in snapshot.iter() {
        for q in obj.questions().iter() {
            let mut cursor = 0.0_f64;
            for a in q.actions().iter() {
                let dur = a.required_time() as f64;
                bars.push(Bar {
                    objective: obj.content().to_string(),
                    question: q.content().to_string(),
                    action: a.content().to_string(),
                    category: a.category().as_str().to_string(),
                    start_hours: cursor,
                    duration_hours: dur,
                    completed: a.completed(),
                });
                cursor += dur;
                if cursor > max_end {
                    max_end = cursor;
                }
            }
        }
    }

    if bars.is_empty() {
        return rsx! {
            section { class: "gantt-view",
                h1 { "Gantt" }
                div { class: "empty",
                    "No actions yet. Bars will appear once questions have actions."
                }
            }
        };
    }

    let total_hours = max_end.ceil().max(1.0);
    let chart_width = total_hours * PX_PER_HOUR;
    let chart_height = bars.len() as f64 * ROW_HEIGHT;

    rsx! {
        section { class: "gantt-view",
            h1 { "Gantt" }
            div { class: "gantt-scroll",
                div {
                    class: "gantt-chart",
                    style: "width: {chart_width}px;",
                    div { class: "gantt-axis",
                        for h in 0..(total_hours as u32) {
                            div {
                                class: "gantt-axis-tick",
                                style: "left: {h as f64 * PX_PER_HOUR}px; width: {PX_PER_HOUR}px;",
                                "{h}h"
                            }
                        }
                    }
                    div {
                        class: "gantt-rows",
                        style: "height: {chart_height}px;",
                        for (i, bar) in bars.into_iter().enumerate() {
                            GanttBar {
                                key: "{i}",
                                row: i,
                                objective: bar.objective,
                                question: bar.question,
                                action: bar.action,
                                category: bar.category,
                                start_hours: bar.start_hours,
                                duration_hours: bar.duration_hours,
                                completed: bar.completed,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GanttBar(
    row: usize,
    objective: String,
    question: String,
    action: String,
    category: String,
    start_hours: f64,
    duration_hours: f64,
    completed: bool,
) -> Element {
    let left = start_hours * PX_PER_HOUR;
    let width = duration_hours.max(0.25) * PX_PER_HOUR;
    let top = row as f64 * ROW_HEIGHT + 4.0;
    let height = ROW_HEIGHT - 8.0;
    let tooltip = format!("{objective} › {question}\n{action} ({category}, {duration_hours}h)");

    rsx! {
        div {
            class: if completed { "gantt-bar done" } else { "gantt-bar" },
            style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px;",
            title: "{tooltip}",
            span { class: "gantt-bar-label", "{action}" }
        }
    }
}
