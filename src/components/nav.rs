use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn Nav() -> Element {
    rsx! {
        nav { class: "nav",
            div { class: "nav-brand", "Smart Manager" }
            div { class: "nav-links",
                Link { to: Route::TodoView {}, class: "nav-link", "Todo" }
                Link { to: Route::GanttView {}, class: "nav-link", "Gantt" }
            }
        }
    }
}
