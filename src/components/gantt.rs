use dioxus::prelude::*;

use crate::app::{AppState, GanttBarData};

const PX_PER_DAY: f64 = 28.0;
const ROW_HEIGHT: f64 = 36.0;

#[component]
pub fn GanttView() -> Element {
    let state = use_context::<AppState>();
    let _view = state.view.read();
    let bars = state.gantt_bars();

    if bars.is_empty() {
        return rsx! {
            section { class: "gantt-view",
                h1 { "Gantt" }
                div { class: "empty",
                    "No objectives with remaining work yet."
                }
            }
        };
    }

    let max_days = bars
        .iter()
        .map(|b| b.days as f64)
        .fold(0.0_f64, f64::max)
        .ceil()
        .max(1.0);
    let chart_width = max_days * PX_PER_DAY + 160.0;
    let chart_height = bars.len() as f64 * ROW_HEIGHT;
    let total_days = max_days as u32;

    rsx! {
        section { class: "gantt-view",
            h1 { "Gantt" }
            div { class: "gantt-scroll",
                div {
                    class: "gantt-chart",
                    style: "width: {chart_width}px;",
                    div { class: "gantt-axis",
                        for d in 0..total_days {
                            div {
                                class: "gantt-axis-tick",
                                style: "left: {160.0 + d as f64 * PX_PER_DAY}px; width: {PX_PER_DAY}px;",
                                "D{d}"
                            }
                        }
                    }
                    div {
                        class: "gantt-rows",
                        style: "height: {chart_height}px;",
                        for (i, bar) in bars.into_iter().enumerate() {
                            GanttBar { key: "{i}", row: i, bar }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GanttBar(row: usize, bar: GanttBarData) -> Element {
    let width = (bar.days as f64).max(0.25) * PX_PER_DAY;
    let top = row as f64 * ROW_HEIGHT + 4.0;
    let height = ROW_HEIGHT - 8.0;
    let left = 160.0_f64;
    let label_top = top;
    let priority_class = bar.priority.to_lowercase();
    let group = bar.group.clone().unwrap_or_else(|| "Ungrouped".to_string());
    let tooltip = format!("{} ({}, {} day(s))", bar.name, bar.priority, bar.days);

    rsx! {
        div {
            class: "gantt-row-label",
            style: "top: {label_top}px; height: {height}px;",
            span { class: "gantt-group", "{group}" }
            span { class: "gantt-name", "{bar.name}" }
        }
        div {
            class: "gantt-bar priority-{priority_class}",
            style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px;",
            title: "{tooltip}",
            span { class: "gantt-bar-label", "{bar.days}d" }
        }
    }
}
