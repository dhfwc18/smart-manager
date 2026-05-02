use super::gantt::{GanttTask, format_days, grouped_sorted};
use crate::priority::Priority;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum TodoFormat {
    Markdown,
    Html,
}

pub fn render(tasks: &[GanttTask], format: TodoFormat) -> String {
    match format {
        TodoFormat::Markdown => render_markdown(tasks),
        TodoFormat::Html => render_html(tasks),
    }
}

pub fn save(tasks: &[GanttTask], format: TodoFormat, path: &Path) -> io::Result<()> {
    let s = render(tasks, format);
    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())
}

fn priority_class(p: Priority) -> &'static str {
    match p {
        Priority::Critical => "priority-critical",
        Priority::High => "priority-high",
        Priority::Medium => "priority-medium",
        Priority::Low => "priority-low",
        Priority::LongTerm => "priority-longterm",
    }
}

include!(concat!(env!("OUT_DIR"), "/todo_md_template.rs"));

fn render_markdown(tasks: &[GanttTask]) -> String {
    let mut s = String::from(TODO_MD_HEAD);
    if tasks.is_empty() {
        s.push_str("(no tasks)\n");
        s.push_str(TODO_MD_TAIL);
        return s;
    }
    for (section, section_tasks) in grouped_sorted(tasks) {
        s.push_str(&format!("## {}\n\n", section));
        for t in section_tasks {
            s.push_str(&format!(
                "- [ ] **[{}]** {} ({}d)\n",
                t.priority.label(),
                t.name,
                format_days(t.days)
            ));
        }
        s.push('\n');
    }
    s.push_str(TODO_MD_TAIL);
    s
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

include!(concat!(env!("OUT_DIR"), "/todo_html_template.rs"));

fn render_html(tasks: &[GanttTask]) -> String {
    let mut s = String::from(TODO_HTML_HEAD);
    if tasks.is_empty() {
        s.push_str("<p>(no tasks)</p>\n");
        s.push_str(TODO_HTML_TAIL);
        return s;
    }
    for (section, section_tasks) in grouped_sorted(tasks) {
        s.push_str(&format!(
            "<section>\n<h2>{}</h2>\n<ul>\n",
            html_escape(&section)
        ));
        for t in section_tasks {
            s.push_str(&format!(
                "<li><input type=\"checkbox\"><span class=\"priority {}\">{}</span>{}<span class=\"days\">({}d)</span></li>\n",
                priority_class(t.priority),
                t.priority.label(),
                html_escape(&t.name),
                format_days(t.days)
            ));
        }
        s.push_str("</ul>\n</section>\n");
    }
    s.push_str(TODO_HTML_TAIL);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(name: &str, days: f32, priority: Priority, group: Option<&str>) -> GanttTask {
        GanttTask::new(name, days, priority, group.map(|g| g.to_string()))
    }

    #[test]
    fn test_render_markdown_with_no_tasks_shows_empty_message() {
        let out = render(&[], TodoFormat::Markdown);
        assert!(out.starts_with("# TODO"));
        assert!(out.contains("(no tasks)"));
    }

    #[test]
    fn test_render_markdown_emits_h2_per_section() {
        let tasks = vec![
            task("A", 1.0, Priority::High, Some("work")),
            task("B", 1.0, Priority::Low, Some("home")),
        ];
        let out = render(&tasks, TodoFormat::Markdown);
        assert!(out.contains("## work"));
        assert!(out.contains("## home"));
    }

    #[test]
    fn test_render_markdown_emits_unchecked_box_for_each_task() {
        let tasks = vec![task("Foo", 2.0, Priority::Medium, Some("g"))];
        let out = render(&tasks, TodoFormat::Markdown);
        assert!(out.contains("- [ ] **[Medium]** Foo (2d)"));
    }

    #[test]
    fn test_render_markdown_orders_within_section_by_priority_desc() {
        let tasks = vec![
            task("LowFirst", 1.0, Priority::Low, Some("g")),
            task("CritSecond", 1.0, Priority::Critical, Some("g")),
        ];
        let out = render(&tasks, TodoFormat::Markdown);
        let crit = out.find("CritSecond").unwrap();
        let low = out.find("LowFirst").unwrap();
        assert!(crit < low);
    }

    #[test]
    fn test_render_markdown_with_no_group_uses_ungrouped() {
        let tasks = vec![task("Floater", 2.0, Priority::Medium, None)];
        let out = render(&tasks, TodoFormat::Markdown);
        assert!(out.contains("## Ungrouped"));
    }

    #[test]
    fn test_render_html_with_no_tasks_emits_doctype_and_empty_message() {
        let out = render(&[], TodoFormat::Html);
        assert!(out.to_lowercase().starts_with("<!doctype html>"));
        assert!(out.contains("(no tasks)"));
        assert!(out.trim_end().ends_with("</html>"));
    }

    #[test]
    fn test_render_html_emits_checkbox_input() {
        let tasks = vec![task("Foo", 1.0, Priority::High, Some("work"))];
        let out = render(&tasks, TodoFormat::Html);
        assert!(out.contains("<input type=\"checkbox\">"));
        assert!(out.contains("Foo"));
    }

    #[test]
    fn test_render_html_uses_priority_class() {
        let tasks = vec![task("X", 1.0, Priority::Critical, Some("g"))];
        let out = render(&tasks, TodoFormat::Html);
        assert!(out.contains("priority-critical"));
    }

    #[test]
    fn test_render_html_escapes_unsafe_characters_in_name() {
        let tasks = vec![task(
            "<script>alert(\"x\")</script>",
            1.0,
            Priority::Low,
            None,
        )];
        let out = render(&tasks, TodoFormat::Html);
        assert!(!out.contains("<script>"));
        assert!(out.contains("&lt;script&gt;"));
        assert!(out.contains("&quot;x&quot;"));
    }

    #[test]
    fn test_render_html_escapes_section_label() {
        let tasks = vec![task("X", 1.0, Priority::Low, Some("a&b"))];
        let out = render(&tasks, TodoFormat::Html);
        assert!(out.contains("<h2>a&amp;b</h2>"));
    }

    #[test]
    fn test_save_writes_file_with_rendered_contents() {
        let path = std::env::temp_dir().join("smart_manager_todo_test.md");
        let _ = std::fs::remove_file(&path);
        let tasks = vec![task("Foo", 1.0, Priority::High, None)];
        save(&tasks, TodoFormat::Markdown, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("# TODO"));
        assert!(contents.contains("- [ ] **[High]** Foo (1d)"));
        let _ = std::fs::remove_file(&path);
    }
}
