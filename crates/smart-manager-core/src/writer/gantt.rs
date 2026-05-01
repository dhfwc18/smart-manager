use crate::priority::Priority;
use comfy_table::Table;
use comfy_table::presets::ASCII_FULL;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub(super) const UNGROUPED: &str = "Ungrouped";
const BAR_WIDTH: usize = 40;

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

pub fn render(tasks: &[GanttTask], format: GanttFormat) -> String {
    match format {
        GanttFormat::Ascii => render_ascii(tasks),
        GanttFormat::Markdown => render_markdown(tasks),
        GanttFormat::Json => render_json(tasks),
    }
}

pub fn save(tasks: &[GanttTask], format: GanttFormat, path: &Path) -> io::Result<()> {
    let s = render(tasks, format);
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
    let n = n.max(1).min(BAR_WIDTH);
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

const MERMAID_FRONTMATTER: &str = "---
config:
  theme: base
  themeVariables:
    taskBkgColor: '#1f3a5f'
    taskBorderColor: '#000000'
    taskTextColor: '#ffffff'
    taskTextLightColor: '#ffffff'
    taskTextDarkColor: '#ffffff'
    taskTextOutsideColor: '#ffffff'
    critBkgColor: '#7a1f1f'
    critBorderColor: '#000000'
    activeTaskBkgColor: '#1f5a4a'
    activeTaskBorderColor: '#000000'
    doneTaskBkgColor: '#1f3a5f'
    doneTaskBorderColor: '#000000'
    sectionBkgColor: '#2a2a2a'
    altSectionBkgColor: '#1a1a1a'
    sectionBkgColor2: '#3a3a3a'
    gridColor: '#888888'
    titleColor: '#ffffff'
    textColor: '#ffffff'
    primaryTextColor: '#ffffff'
    todayLineColor: '#ff6b6b'
---
";

const GANTT_BASE_DATE: &str = "2025-01-01";

fn render_markdown(tasks: &[GanttTask]) -> String {
    let mut s = format!(
        "```mermaid\n{}gantt\n    title Schedule\n    dateFormat YYYY-MM-DD\n    axisFormat %m-%d\n",
        MERMAID_FRONTMATTER
    );
    if tasks.is_empty() {
        s.push_str("```\n");
        return s;
    }
    let mut id_counter: usize = 0;
    for (section, section_tasks) in grouped_sorted(tasks) {
        s.push_str(&format!("    section {}\n", sanitize_mermaid(&section)));
        let mut prev_id: Option<String> = None;
        for t in section_tasks {
            let tid = format!("t{}", id_counter);
            id_counter += 1;
            let modifier = if matches!(t.priority, Priority::Critical) {
                "crit, "
            } else {
                ""
            };
            let start = match &prev_id {
                Some(p) => format!("after {}", p),
                None => GANTT_BASE_DATE.to_string(),
            };
            s.push_str(&format!(
                "    {} :{}{}, {}, {}d\n",
                sanitize_mermaid(&t.name),
                modifier,
                tid,
                start,
                format_days(t.days)
            ));
            prev_id = Some(tid);
        }
    }
    s.push_str("```\n");
    s
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

    fn task(name: &str, days: f32, priority: Priority, group: Option<&str>) -> GanttTask {
        GanttTask::new(name, days, priority, group.map(|g| g.to_string()))
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
        let tasks = vec![task("A", 1.0, Priority::High, Some("work"))];
        let out = render(&tasks, GanttFormat::Ascii);
        assert!(out.contains("[work]"));
        assert!(out.contains("A"));
    }

    #[test]
    fn test_render_ascii_with_no_group_uses_ungrouped() {
        let tasks = vec![task("Floater", 2.0, Priority::Medium, None)];
        let out = render(&tasks, GanttFormat::Ascii);
        assert!(out.contains("[Ungrouped]"));
    }

    #[test]
    fn test_render_ascii_orders_sections_by_priority_score_sum_desc() {
        let tasks = vec![
            task("Low task", 1.0, Priority::Low, Some("alpha")),
            task("Crit task", 1.0, Priority::Critical, Some("beta")),
        ];
        let out = render(&tasks, GanttFormat::Ascii);
        let beta = out.find("[beta]").expect("beta header missing");
        let alpha = out.find("[alpha]").expect("alpha header missing");
        assert!(beta < alpha, "higher priority section should appear first");
    }

    #[test]
    fn test_render_ascii_orders_within_section_by_priority_desc() {
        let tasks = vec![
            task("LowFirst", 1.0, Priority::Low, Some("g")),
            task("CritSecond", 1.0, Priority::Critical, Some("g")),
        ];
        let out = render(&tasks, GanttFormat::Ascii);
        let crit = out.find("CritSecond").unwrap();
        let low = out.find("LowFirst").unwrap();
        assert!(crit < low);
    }

    #[test]
    fn test_render_ascii_uses_only_ascii_chars() {
        let tasks = vec![task("A", 2.5, Priority::High, Some("work"))];
        let out = render(&tasks, GanttFormat::Ascii);
        assert!(out.is_ascii(), "ASCII output must contain only ASCII");
    }

    #[test]
    fn test_render_markdown_emits_mermaid_block() {
        let tasks = vec![task("A", 3.0, Priority::Medium, Some("work"))];
        let out = render(&tasks, GanttFormat::Markdown);
        assert!(out.starts_with("```mermaid\n"));
        assert!(out.contains("\ngantt\n"));
        assert!(out.contains("section work"));
        assert!(out.trim_end().ends_with("```"));
    }

    #[test]
    fn test_render_markdown_critical_uses_crit_modifier() {
        let tasks = vec![task("A", 1.0, Priority::Critical, Some("g"))];
        let out = render(&tasks, GanttFormat::Markdown);
        assert!(out.contains(":crit, t0"));
    }

    #[test]
    fn test_render_markdown_chains_subsequent_tasks_with_after() {
        let tasks = vec![
            task("First", 2.0, Priority::High, Some("g")),
            task("Second", 1.0, Priority::High, Some("g")),
        ];
        let out = render(&tasks, GanttFormat::Markdown);
        assert!(out.contains(":t0, 2025-01-01, 2d"));
        assert!(out.contains("after t0, 1d"));
    }

    #[test]
    fn test_render_markdown_sanitizes_colons_in_name() {
        let tasks = vec![task("A: bad name", 1.0, Priority::Low, Some("g"))];
        let out = render(&tasks, GanttFormat::Markdown);
        assert!(!out.contains("A: bad"));
        assert!(out.contains("A  bad name"));
    }

    #[test]
    fn test_render_json_with_no_tasks_emits_empty_array() {
        let out = render(&[], GanttFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["tasks"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_render_json_roundtrips_through_serde() {
        let tasks = vec![
            task("Foo", 2.0, Priority::High, Some("work")),
            task("Bar", 0.5, Priority::Low, None),
        ];
        let out = render(&tasks, GanttFormat::Json);
        let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
        let arr = doc["tasks"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Foo");
        assert_eq!(arr[0]["days"], 2.0);
        assert_eq!(arr[0]["priority"], "High");
        assert_eq!(arr[0]["group"], "work");
        assert!(arr[1]["group"].is_null());
    }

    #[test]
    fn test_render_json_escapes_quotes_in_name() {
        let tasks = vec![task("Has \"quotes\"", 1.0, Priority::Low, None)];
        let out = render(&tasks, GanttFormat::Json);
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
        let tasks = vec![task("Foo", 1.0, Priority::High, None)];
        save(&tasks, GanttFormat::Ascii, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("Foo"));
        let _ = std::fs::remove_file(&path);
    }
}
