use super::gantt::format_days;
use crate::priority::Priority;
use crate::questions::Objective;
use crate::writer::{ObjectiveSchedule, ScheduledQuestion, objectives_to_schedule};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum TodoFormat {
    Markdown,
    Html,
}

include!(concat!(env!("OUT_DIR"), "/todo_md_template.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_html_template.rs"));

pub fn render(objectives: &[Objective], format: TodoFormat) -> String {
    match format {
        TodoFormat::Markdown => render_markdown(objectives),
        TodoFormat::Html => render_html(objectives),
    }
}

pub fn save(objectives: &[Objective], format: TodoFormat, path: &Path) -> io::Result<()> {
    let s = render(objectives, format);
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

fn render_markdown(objectives: &[Objective]) -> String {
    let schedule = objectives_to_schedule(objectives);
    let mut s = String::from(TODO_MD_HEAD);

    if schedule.iter().all(|o| o.questions.is_empty()) {
        s.push_str("(no tasks)\n");
        s.push_str(TODO_MD_TAIL);
        return s;
    }

    let mut first = true;
    for sched in &schedule {
        if sched.questions.is_empty() {
            continue;
        }
        if !first {
            s.push('\n');
        }
        first = false;
        s.push_str(&render_markdown_objective(sched));
    }
    s.push_str(TODO_MD_TAIL);
    s
}

fn render_markdown_objective(sched: &ObjectiveSchedule) -> String {
    let mut s = String::new();
    let header = if sched.met {
        format!("## ~~{}~~ (met)\n\n", sched.objective)
    } else {
        format!("## {}\n\n", sched.objective)
    };
    s.push_str(&header);
    for q in &sched.questions {
        s.push_str(&render_markdown_question(q));
    }
    s
}

fn render_markdown_question(q: &ScheduledQuestion) -> String {
    let mut s = String::new();
    let qbox = if q.answered { "[x]" } else { "[ ]" };
    s.push_str(&format!(
        "- {} **[{}]** [Q{}] {} ({}d)\n",
        qbox,
        q.priority.label(),
        q.id,
        q.content,
        format_days(q.days),
    ));
    for a in &q.actions {
        let abox = if a.completed { "[x]" } else { "[ ]" };
        s.push_str(&format!(
            "  - {} {} ({}d, {})\n",
            abox,
            a.content,
            format_days(a.days),
            a.category,
        ));
    }
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

fn checkbox_attr(checked: bool) -> &'static str {
    if checked { " checked" } else { "" }
}

fn render_html(objectives: &[Objective]) -> String {
    let schedule = objectives_to_schedule(objectives);
    let mut s = String::from(TODO_HTML_HEAD);

    if schedule.iter().all(|o| o.questions.is_empty()) {
        s.push_str("<p>(no tasks)</p>\n");
        s.push_str(TODO_HTML_TAIL);
        return s;
    }

    for sched in &schedule {
        if sched.questions.is_empty() {
            continue;
        }
        s.push_str(&render_html_objective(sched));
    }
    s.push_str(TODO_HTML_TAIL);
    s
}

fn render_html_objective(sched: &ObjectiveSchedule) -> String {
    let mut s = String::from("<section>\n");
    if sched.met {
        s.push_str(&format!(
            "<h2><s>{}</s> (met)</h2>\n",
            html_escape(&sched.objective)
        ));
    } else {
        s.push_str(&format!("<h2>{}</h2>\n", html_escape(&sched.objective)));
    }
    s.push_str("<ul>\n");
    for q in &sched.questions {
        s.push_str(&render_html_question(q));
    }
    s.push_str("</ul>\n</section>\n");
    s
}

fn render_html_question(q: &ScheduledQuestion) -> String {
    let mut s = String::from("<li>");
    s.push_str(&format!(
        "<input type=\"checkbox\"{}><span class=\"priority {}\">{}</span>[Q{}] {}<span class=\"days\">({}d)</span>",
        checkbox_attr(q.answered),
        priority_class(q.priority),
        q.priority.label(),
        q.id,
        html_escape(&q.content),
        format_days(q.days),
    ));
    if !q.actions.is_empty() {
        s.push_str("\n<ul>\n");
        for a in &q.actions {
            s.push_str(&format!(
                "<li><input type=\"checkbox\"{}>{}<span class=\"days\">({}d, {})</span></li>\n",
                checkbox_attr(a.completed),
                html_escape(&a.content),
                format_days(a.days),
                a.category,
            ));
        }
        s.push_str("</ul>\n");
    }
    s.push_str("</li>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questions::{ActionCategory, ActionPoint, QuestionPriority};

    fn obj(content: &str) -> Objective {
        Objective::new(content.into())
    }

    #[test]
    fn test_render_markdown_with_no_objectives_shows_empty_message() {
        let out = render(&[], TodoFormat::Markdown);
        assert!(out.starts_with("# TODOs"));
        assert!(out.contains("(no tasks)"));
    }

    #[test]
    fn test_render_markdown_uses_objective_as_h2_subheading() {
        let mut a = obj("Ship MVP");
        a.add_question("q".into(), QuestionPriority::High, None);
        let mut b = obj("Refresh docs");
        b.add_question("q".into(), QuestionPriority::Low, None);
        let out = render(&[a, b], TodoFormat::Markdown);
        assert!(out.contains("## Ship MVP"));
        assert!(out.contains("## Refresh docs"));
    }

    #[test]
    fn test_render_markdown_emits_question_with_clickable_dash_box_and_id() {
        let mut o = obj("Obj");
        o.add_question("Find pilot".into(), QuestionPriority::Critical, None);
        let out = render(&[o], TodoFormat::Markdown);
        assert!(out.contains("- [ ] **[Critical]** [Q0] Find pilot (0d)"));
    }

    #[test]
    fn test_render_markdown_nests_actions_under_question() {
        let mut o = obj("Obj");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        q.push_action(ActionPoint::new(
            "design".into(),
            ActionCategory::Analysis,
            2.0,
        ));
        q.push_action(ActionPoint::new(
            "write".into(),
            ActionCategory::Writing,
            1.0,
        ));
        let out = render(&[o], TodoFormat::Markdown);
        assert!(out.contains("  - [ ] design (2d, analysis)"));
        assert!(out.contains("  - [ ] write (1d, writing)"));
    }

    #[test]
    fn test_render_markdown_checks_completed_action_and_answered_question() {
        let mut o = obj("Obj");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        let mut a = ActionPoint::new("done".into(), ActionCategory::Writing, 1.0);
        a.set_completed(true);
        q.push_action(a);
        q.set_answered(true).unwrap();
        let out = render(&[o], TodoFormat::Markdown);
        assert!(out.contains("- [x] **[Medium]** [Q0] q"));
        assert!(out.contains("  - [x] done"));
    }

    #[test]
    fn test_render_markdown_strikes_met_objective_heading() {
        let mut o = obj("Already done");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        q.set_answered(true).unwrap();
        let _ = o.set_met(true);
        let out = render(&[o], TodoFormat::Markdown);
        assert!(out.contains("## ~~Already done~~ (met)"));
    }

    #[test]
    fn test_render_html_with_no_objectives_emits_doctype_and_empty_message() {
        let out = render(&[], TodoFormat::Html);
        assert!(out.to_lowercase().starts_with("<!doctype html>"));
        assert!(out.contains("(no tasks)"));
        assert!(out.trim_end().ends_with("</html>"));
    }

    #[test]
    fn test_render_html_emits_h2_per_objective() {
        let mut o = obj("Ship & MVP");
        o.add_question("q".into(), QuestionPriority::High, None);
        let out = render(&[o], TodoFormat::Html);
        assert!(out.contains("<h2>Ship &amp; MVP</h2>"));
    }

    #[test]
    fn test_render_html_nests_actions_inside_question_li() {
        let mut o = obj("Obj");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        q.push_action(ActionPoint::new(
            "step".into(),
            ActionCategory::Writing,
            1.0,
        ));
        let out = render(&[o], TodoFormat::Html);
        // Outer <li> contains a nested <ul> with one inner <li>.
        assert!(out.contains("<ul>\n<li><input type=\"checkbox\">step"));
    }

    #[test]
    fn test_render_html_marks_completed_action_with_checked() {
        let mut o = obj("Obj");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        let mut done = ActionPoint::new("done".into(), ActionCategory::Writing, 1.0);
        done.set_completed(true);
        q.push_action(done);
        let out = render(&[o], TodoFormat::Html);
        assert!(out.contains("<input type=\"checkbox\" checked>done"));
    }

    #[test]
    fn test_render_html_marks_answered_question_with_checked() {
        let mut o = obj("Obj");
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        q.set_answered(true).unwrap();
        let out = render(&[o], TodoFormat::Html);
        assert!(out.contains("<input type=\"checkbox\" checked>"));
    }

    #[test]
    fn test_render_html_uses_priority_class() {
        let mut o = obj("Obj");
        o.add_question("q".into(), QuestionPriority::Critical, None);
        let out = render(&[o], TodoFormat::Html);
        assert!(out.contains("priority-critical"));
    }

    #[test]
    fn test_render_html_escapes_unsafe_characters_in_question_content() {
        let mut o = obj("Obj");
        o.add_question(
            "<script>alert(\"x\")</script>".into(),
            QuestionPriority::Low,
            None,
        );
        let out = render(&[o], TodoFormat::Html);
        assert!(!out.contains("<script>"));
        assert!(out.contains("&lt;script&gt;"));
        assert!(out.contains("&quot;x&quot;"));
    }

    #[test]
    fn test_save_writes_file_with_rendered_contents() {
        let path = std::env::temp_dir().join("smart_manager_todo_test.md");
        let _ = std::fs::remove_file(&path);
        let mut o = obj("Foo");
        o.add_question("q".into(), QuestionPriority::High, None);
        save(&[o], TodoFormat::Markdown, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("# TODOs"));
        assert!(contents.contains("## Foo"));
        assert!(contents.contains("- [ ] **[High]** [Q0] q"));
        let _ = std::fs::remove_file(&path);
    }
}
