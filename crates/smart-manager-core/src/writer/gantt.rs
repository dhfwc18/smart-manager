use crate::priority::Priority;
use crate::questions::Objective;
use crate::writer::{ObjectiveSchedule, ScheduledQuestion, objectives_to_schedule};
use comfy_table::Table;
use comfy_table::presets::ASCII_FULL;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub(super) const UNGROUPED: &str = "Ungrouped";
const BAR_WIDTH: usize = 40;
const GANTT_BASE_DATE: &str = "2025-01-01";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttTask {
    pub name: String,
    pub days: f32,
    pub priority: Priority,
    pub group: Option<String>,
}

impl GanttTask {
    pub fn new(
        name: impl Into<String>,
        days: f32,
        priority: Priority,
        group: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            days,
            priority,
            group,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GanttFormat {
    Ascii,
    Markdown,
    Json,
}

include!(concat!(env!("OUT_DIR"), "/gantt_chart_md_template.rs"));
include!(concat!(env!("OUT_DIR"), "/gantt_table_md_template.rs"));

pub fn render(objectives: &[Objective], format: GanttFormat) -> String {
    match format {
        GanttFormat::Ascii => render_ascii(&super::objectives_to_gantt_tasks(objectives)),
        GanttFormat::Markdown => render_markdown(objectives),
        GanttFormat::Json => render_json(&super::objectives_to_gantt_tasks(objectives)),
    }
}

pub fn save(objectives: &[Objective], format: GanttFormat, path: &Path) -> io::Result<()> {
    let s = render(objectives, format);
    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())
}

pub(super) fn group_label(t: &GanttTask) -> &str {
    t.group.as_deref().unwrap_or(UNGROUPED)
}

pub(super) fn grouped_sorted(tasks: &[GanttTask]) -> Vec<(String, Vec<&GanttTask>)> {
    let mut order: Vec<String> = Vec::new();
    let mut buckets: HashMap<String, Vec<&GanttTask>> = HashMap::new();
    for t in tasks {
        let key = group_label(t).to_string();
        if !buckets.contains_key(&key) {
            order.push(key.clone());
        }
        buckets.entry(key).or_default().push(t);
    }
    let mut sections: Vec<(String, Vec<&GanttTask>)> = order
        .into_iter()
        .map(|k| {
            let v = buckets.remove(&k).unwrap();
            (k, v)
        })
        .collect();
    for (_, v) in sections.iter_mut() {
        v.sort_by(|a, b| b.priority.score().cmp(&a.priority.score()));
    }
    sections.sort_by(|a, b| {
        let sa: u32 = a.1.iter().map(|t| t.priority.score() as u32).sum();
        let sb: u32 = b.1.iter().map(|t| t.priority.score() as u32).sum();
        sb.cmp(&sa)
    });
    sections
}

pub(super) fn format_days(days: f32) -> String {
    if !days.is_finite() {
        return "0".to_string();
    }
    if (days - days.trunc()).abs() < f32::EPSILON {
        format!("{}", days as i64)
    } else {
        format!("{:.1}", days)
    }
}

fn make_bar(days: f32, max_days: f32) -> String {
    if max_days <= 0.0 || !days.is_finite() || days <= 0.0 {
        return String::new();
    }
    let n = ((days / max_days) * BAR_WIDTH as f32).round() as usize;
    let n = n.clamp(1, BAR_WIDTH);
    "#".repeat(n)
}

fn render_ascii(tasks: &[GanttTask]) -> String {
    let mut out = String::from("SCHEDULE\n========\n\n");
    if tasks.is_empty() {
        out.push_str("(no tasks)\n");
        return out;
    }
    let max_days = tasks.iter().map(|t| t.days).fold(0.0_f32, f32::max);
    for (section, section_tasks) in grouped_sorted(tasks) {
        out.push_str(&format!("[{}]\n", section));
        let mut table = Table::new();
        table.load_preset(ASCII_FULL);
        table.set_header(vec!["Pri", "Name", "Days", "Bar"]);
        for t in section_tasks {
            table.add_row(vec![
                t.priority.letter().to_string(),
                t.name.clone(),
                format_days(t.days),
                make_bar(t.days, max_days),
            ]);
        }
        out.push_str(&table.to_string());
        out.push_str("\n\n");
    }
    out
}

fn sanitize_mermaid(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ':' | ',' | '\n' | '\r' => ' ',
            _ => c,
        })
        .collect()
}

fn render_markdown(objectives: &[Objective]) -> String {
    let schedule = objectives_to_schedule(objectives);
    let has_any = schedule.iter().any(|s| !s.questions.is_empty());

    let mut out = String::from(GANTT_CHART_MD_HEAD);
    if has_any {
        out.push_str(&render_chart_body(&schedule));
    }
    out.push_str(GANTT_CHART_MD_TAIL);

    out.push_str(GANTT_TABLE_MD_HEAD);
    out.push_str(&render_table_body(&schedule));
    out.push_str(GANTT_TABLE_MD_TAIL);

    out
}

fn render_chart_body(schedule: &[ObjectiveSchedule]) -> String {
    let mut out = String::new();
    let mut tid_counter: usize = 0;
    for sched in schedule {
        if sched.questions.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "    section {}\n",
            sanitize_mermaid(&sched.objective)
        ));
        let mut tid_for_qid: HashMap<usize, String> = HashMap::new();
        let mut prev_tid: Option<String> = None;
        for q in &sched.questions {
            let tid = format!("t{}", tid_counter);
            tid_counter += 1;

            let mut tags: Vec<&str> = Vec::new();
            if matches!(q.priority, Priority::Critical) {
                tags.push("crit");
            }
            if q.answered {
                tags.push("done");
            }
            let tag_prefix = if tags.is_empty() {
                String::new()
            } else {
                format!("{}, ", tags.join(", "))
            };

            let start = q
                .prereq
                .and_then(|pid| tid_for_qid.get(&pid).cloned())
                .map(|p| format!("after {}", p))
                .unwrap_or_else(|| match &prev_tid {
                    Some(p) => format!("after {}", p),
                    None => GANTT_BASE_DATE.to_string(),
                });

            let label = format!("[Q{}] {}", q.id, sanitize_mermaid(&q.content));
            let days = format_days(q.days.max(0.0));
            out.push_str(&format!(
                "    {} :{}{}, {}, {}d\n",
                label, tag_prefix, tid, start, days
            ));

            tid_for_qid.insert(q.id, tid.clone());
            prev_tid = Some(tid);
        }
    }
    out
}

fn escape_table(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn render_table_body(schedule: &[ObjectiveSchedule]) -> String {
    let mut out = String::new();
    let mut first = true;
    for sched in schedule {
        if sched.questions.is_empty() {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        let header = if sched.met {
            format!("### ~~{}~~ (met)\n\n", escape_table(&sched.objective))
        } else {
            format!("### {}\n\n", escape_table(&sched.objective))
        };
        out.push_str(&header);
        out.push_str("| ID | Question | Priority | Days | Status | Action points |\n");
        out.push_str("|----|----------|----------|------|--------|---------------|\n");
        for q in &sched.questions {
            out.push_str(&format_table_row(q));
        }
    }
    out
}

fn format_table_row(q: &ScheduledQuestion) -> String {
    let qtext = if q.answered {
        format!("~~{}~~", escape_table(&q.content))
    } else {
        escape_table(&q.content)
    };
    let status = if q.answered { "done" } else { "open" };
    let actions = if q.actions.is_empty() {
        "—".to_string()
    } else {
        q.actions
            .iter()
            .map(|a| {
                let check = if a.completed { "[x]" } else { "[ ]" };
                format!(
                    "{} {} ({}d, {})",
                    check,
                    escape_table(&a.content),
                    format_days(a.days),
                    a.category
                )
            })
            .collect::<Vec<_>>()
            .join("<br>")
    };
    format!(
        "| Q{} | {} | {} | {} | {} | {} |\n",
        q.id,
        qtext,
        q.priority.label(),
        format_days(q.days),
        status,
        actions
    )
}

#[derive(Serialize)]
struct GanttDoc<'a> {
    tasks: &'a [GanttTask],
}

fn render_json(tasks: &[GanttTask]) -> String {
    let doc = GanttDoc { tasks };
    serde_json::to_string_pretty(&doc).expect("gantt tasks should always serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questions::{ActionCategory, ActionPoint, QuestionPriority, Tag};

    fn task(name: &str, days: f32, priority: Priority, group: Option<&str>) -> GanttTask {
        GanttTask::new(name, days, priority, group.map(|g| g.to_string()))
    }

    fn obj_with_question(
        name: &str,
        q_text: &str,
        priority: QuestionPriority,
        days: f32,
    ) -> Objective {
        let mut o = Objective::new(name.into());
        let q = o.add_question(q_text.into(), priority, None);
        q.push_action(ActionPoint::new("a".into(), ActionCategory::Writing, days));
        o
    }

    #[test]
    fn test_format_days_integer_omits_decimal() {
        assert_eq!(format_days(3.0), "3");
    }

    #[test]
    fn test_format_days_fractional_keeps_one_decimal() {
        assert_eq!(format_days(2.5), "2.5");
    }

    #[test]
    fn test_make_bar_zero_max_returns_empty() {
        assert_eq!(make_bar(1.0, 0.0), "");
    }

    #[test]
    fn test_make_bar_full_length_for_max_value() {
        let bar = make_bar(10.0, 10.0);
        assert_eq!(bar.len(), BAR_WIDTH);
        assert!(bar.chars().all(|c| c == '#'));
    }

    #[test]
    fn test_render_ascii_with_no_tasks_shows_empty_message() {
        let out = render(&[], GanttFormat::Ascii);
        assert!(out.contains("(no tasks)"));
    }

    #[test]
    fn test_render_ascii_includes_section_label() {
        let mut o = obj_with_question("X", "q", QuestionPriority::High, 1.0);
        o.add_tag(Tag::new("work"));
        let out = render(&[o], GanttFormat::Ascii);
        assert!(out.contains("[work]"));
    }

    #[test]
    fn test_render_ascii_with_no_group_uses_ungrouped() {
        let o = obj_with_question("Floater", "q", QuestionPriority::Medium, 2.0);
        let out = render(&[o], GanttFormat::Ascii);
        assert!(out.contains("[Ungrouped]"));
    }

    #[test]
    fn test_render_ascii_orders_within_section_by_priority_desc() {
        let tasks = vec![
            task("LowFirst", 1.0, Priority::Low, Some("g")),
            task("CritSecond", 1.0, Priority::Critical, Some("g")),
        ];
        let out = render_ascii(&tasks);
        let crit = out.find("CritSecond").unwrap();
        let low = out.find("LowFirst").unwrap();
        assert!(crit < low);
    }

    #[test]
    fn test_render_ascii_uses_only_ascii_chars() {
        let mut o = obj_with_question("Foo", "q", QuestionPriority::High, 2.5);
        o.add_tag(Tag::new("work"));
        let out = render(&[o], GanttFormat::Ascii);
        assert!(out.is_ascii(), "ASCII output must contain only ASCII");
    }

    #[test]
    fn test_render_markdown_emits_mermaid_block() {
        let o = obj_with_question("Ship MVP", "q", QuestionPriority::Medium, 3.0);
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.starts_with("```mermaid\n"));
        assert!(out.contains("\ngantt\n"));
        assert!(out.contains("section Ship MVP"));
    }

    #[test]
    fn test_render_markdown_includes_question_id_label_on_bars() {
        let o = obj_with_question("Obj", "What is X", QuestionPriority::Medium, 2.0);
        let out = render(&[o], GanttFormat::Markdown);
        assert!(
            out.contains("[Q0] What is X"),
            "expected [Q0] label, got: {out}"
        );
    }

    #[test]
    fn test_render_markdown_critical_question_gets_crit_tag() {
        let o = obj_with_question("Obj", "q", QuestionPriority::Critical, 1.0);
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains(":crit, "));
    }

    #[test]
    fn test_render_markdown_answered_question_gets_done_tag() {
        let mut o = Objective::new("Obj".into());
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        let mut a = ActionPoint::new("a".into(), ActionCategory::Writing, 1.0);
        a.set_completed(true);
        q.push_action(a);
        q.set_answered(true).unwrap();
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains("done, "), "expected done tag, got: {out}");
    }

    #[test]
    fn test_render_markdown_includes_reference_table() {
        let o = obj_with_question("Obj", "q", QuestionPriority::Medium, 1.0);
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains("## Reference"));
        assert!(out.contains("### Obj"));
        assert!(out.contains("| ID |"));
        assert!(out.contains("| Q0 |"));
    }

    #[test]
    fn test_render_markdown_orders_by_priority_then_singletons_first() {
        let mut o = Objective::new("Obj".into());
        // Two High-priority questions: a chain (medium prereq -> high leaf) and a solo high.
        let med_id = o
            .add_question("Med".into(), QuestionPriority::Medium, None)
            .id();
        let _high_with_prereq = o
            .add_question("HighWithPrereq".into(), QuestionPriority::High, Some(med_id))
            .id();
        let _high_solo = o
            .add_question("HighSolo".into(), QuestionPriority::High, None)
            .id();

        let out = render(&[o], GanttFormat::Markdown);
        let solo = out.find("[Q2] HighSolo").expect("solo missing");
        let med = out.find("[Q0] Med").expect("med missing");
        let leaf = out.find("[Q1] HighWithPrereq").expect("leaf missing");
        assert!(
            solo < med && med < leaf,
            "expected solo high then med (prereq) then high leaf, got order: solo={solo} med={med} leaf={leaf}"
        );
    }

    #[test]
    fn test_render_markdown_chains_dependent_via_after_prereq() {
        let mut o = Objective::new("Obj".into());
        let p = o
            .add_question("Root".into(), QuestionPriority::Medium, None)
            .id();
        o.add_question("Leaf".into(), QuestionPriority::High, Some(p));
        let out = render(&[o], GanttFormat::Markdown);
        // Root emitted first as t0, Leaf as t1 with `after t0`.
        assert!(out.contains(":t0, 2025-01-01"));
        assert!(out.contains("after t0, "));
    }

    #[test]
    fn test_render_markdown_sanitizes_colons_in_question_text() {
        let o = obj_with_question("Obj", "A: bad name", QuestionPriority::Low, 1.0);
        let out = render(&[o], GanttFormat::Markdown);
        // The chart row replaces ':' with ' '; the table row escapes table-breaking
        // chars but preserves ':' since it does not break tables.
        assert!(out.contains("[Q0] A  bad name"));
    }

    #[test]
    fn test_render_markdown_table_strikes_answered_question() {
        let mut o = Objective::new("Obj".into());
        let q = o.add_question("Done thing".into(), QuestionPriority::Medium, None);
        q.set_answered(true).unwrap();
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains("~~Done thing~~"));
    }

    #[test]
    fn test_render_markdown_table_marks_action_completion_with_checkbox_only() {
        let mut o = Objective::new("Obj".into());
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        let mut done = ActionPoint::new("done step".into(), ActionCategory::Writing, 1.0);
        done.set_completed(true);
        q.push_action(done);
        q.push_action(ActionPoint::new(
            "open step".into(),
            ActionCategory::Writing,
            1.0,
        ));
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains("[x] done step"));
        assert!(out.contains("[ ] open step"));
        assert!(
            !out.contains("~~[x]"),
            "completed action should not be wrapped in strikethrough"
        );
    }

    #[test]
    fn test_render_markdown_table_marks_met_objective() {
        let mut o = Objective::new("Done obj".into());
        let q = o.add_question("q".into(), QuestionPriority::Medium, None);
        q.set_answered(true).unwrap();
        o.set_met(true).unwrap();
        let out = render(&[o], GanttFormat::Markdown);
        assert!(out.contains("~~Done obj~~ (met)"));
    }

    #[test]
    fn test_render_json_with_no_tasks_emits_empty_array() {
        let out = render(&[], GanttFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["tasks"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_render_json_includes_objective_summary_per_objective() {
        let o = obj_with_question("Foo", "q", QuestionPriority::High, 2.0);
        let out = render(&[o], GanttFormat::Json);
        let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
        let arr = doc["tasks"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "Foo");
    }

    #[test]
    fn test_render_json_escapes_quotes_in_name() {
        let tasks = vec![task("Has \"quotes\"", 1.0, Priority::Low, None)];
        let out = render_json(&tasks);
        let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(doc["tasks"][0]["name"], "Has \"quotes\"");
    }

    #[test]
    fn test_gantt_task_deserializes_from_json() {
        let json = r#"{"name":"X","days":1.5,"priority":"Critical","group":"g"}"#;
        let t: GanttTask = serde_json::from_str(json).unwrap();
        assert_eq!(t.name, "X");
        assert_eq!(t.days, 1.5);
        assert_eq!(t.priority, Priority::Critical);
        assert_eq!(t.group.as_deref(), Some("g"));
    }

    #[test]
    fn test_save_writes_file_with_rendered_contents() {
        let path = std::env::temp_dir().join("smart_manager_gantt_test.txt");
        let _ = std::fs::remove_file(&path);
        let o = obj_with_question("Foo", "q", QuestionPriority::High, 1.0);
        save(&[o], GanttFormat::Ascii, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("Foo"));
        let _ = std::fs::remove_file(&path);
    }
}
