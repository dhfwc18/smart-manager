pub mod gantt;
pub mod todo;

use crate::priority::Priority;
use crate::questions::Objective;
use crate::writer::gantt::GanttTask;
use std::collections::HashMap;

pub fn objectives_to_gantt_tasks(objectives: &[Objective]) -> Vec<GanttTask> {
    let scores = tag_priority_score_sums(objectives);
    objectives
        .iter()
        .map(|o| {
            GanttTask::new(
                o.content().to_string(),
                o.remaining_time_needed(),
                objective_max_priority(o),
                pick_group(o, &scores),
            )
        })
        .collect()
}

pub fn tag_priority_score_sums(objectives: &[Objective]) -> HashMap<String, u32> {
    let mut sums: HashMap<String, u32> = HashMap::new();
    for o in objectives {
        let s = objective_priority_score_sum(o);
        for t in o.tags() {
            *sums.entry(t.name().to_string()).or_insert(0) += s;
        }
    }
    sums
}

fn objective_priority_score_sum(o: &Objective) -> u32 {
    o.questions()
        .iter()
        .map(|q| Priority::from(q.priority()).score() as u32)
        .sum()
}

fn objective_max_priority(o: &Objective) -> Priority {
    o.questions()
        .iter()
        .map(|q| Priority::from(q.priority()))
        .max_by_key(|p| p.score())
        .unwrap_or(Priority::Medium)
}

fn pick_group(o: &Objective, scores: &HashMap<String, u32>) -> Option<String> {
    if o.tags().is_empty() {
        return None;
    }
    let mut best: Option<(String, u32)> = None;
    for t in o.tags() {
        let name = t.name();
        let score = scores.get(name).copied().unwrap_or(0);
        let take = match &best {
            None => true,
            Some((b_name, b_score)) => {
                score > *b_score || (score == *b_score && name < b_name.as_str())
            }
        };
        if take {
            best = Some((name.to_string(), score));
        }
    }
    best.map(|(n, _)| n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questions::{QuestionPriority, Tag};

    fn objective(content: &str, priorities: Vec<QuestionPriority>, tags: &[&str]) -> Objective {
        let mut o = Objective::new(content.into());
        for p in priorities {
            o.add_question("q".into(), p, None);
        }
        for t in tags {
            o.add_tag(Tag::new(*t));
        }
        o
    }

    #[test]
    fn test_pick_group_with_no_tags_returns_none() {
        let o = objective("x", vec![QuestionPriority::Medium], &[]);
        let scores = tag_priority_score_sums(std::slice::from_ref(&o));
        assert_eq!(pick_group(&o, &scores), None);
    }

    #[test]
    fn test_pick_group_with_single_tag_returns_that_tag() {
        let o = objective("x", vec![QuestionPriority::Medium], &["work"]);
        let scores = tag_priority_score_sums(std::slice::from_ref(&o));
        assert_eq!(pick_group(&o, &scores), Some("work".to_string()));
    }

    #[test]
    fn test_pick_group_with_two_tags_picks_higher_priority_score_sum() {
        let objectives = vec![
            objective("a", vec![QuestionPriority::Critical], &["work"]),
            objective(
                "b",
                vec![QuestionPriority::Critical, QuestionPriority::High],
                &["work", "home"],
            ),
            objective("c", vec![QuestionPriority::Low], &["home"]),
        ];
        let scores = tag_priority_score_sums(&objectives);
        assert_eq!(
            pick_group(&objectives[1], &scores),
            Some("work".to_string())
        );
    }

    #[test]
    fn test_pick_group_with_tied_scores_picks_alphabetically_first() {
        let objectives = vec![objective(
            "a",
            vec![QuestionPriority::Medium],
            &["zebra", "apple"],
        )];
        let scores = tag_priority_score_sums(&objectives);
        assert_eq!(
            pick_group(&objectives[0], &scores),
            Some("apple".to_string())
        );
    }

    #[test]
    fn test_objectives_to_gantt_tasks_assigns_multi_tag_to_higher_score_group() {
        let objectives = vec![
            objective("Big work", vec![QuestionPriority::Critical], &["work"]),
            objective("Both", vec![QuestionPriority::Medium], &["work", "home"]),
            objective("Just home", vec![QuestionPriority::Low], &["home"]),
        ];
        let tasks = objectives_to_gantt_tasks(&objectives);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[1].name, "Both");
        assert_eq!(tasks[1].group.as_deref(), Some("work"));
    }

    #[test]
    fn test_objectives_to_gantt_tasks_uses_max_question_priority() {
        let objectives = vec![objective(
            "x",
            vec![
                QuestionPriority::Low,
                QuestionPriority::Critical,
                QuestionPriority::Medium,
            ],
            &[],
        )];
        let tasks = objectives_to_gantt_tasks(&objectives);
        assert_eq!(tasks[0].priority, Priority::Critical);
    }

    #[test]
    fn test_objectives_to_gantt_tasks_with_no_tags_uses_none_group() {
        let objectives = vec![objective("x", vec![QuestionPriority::Medium], &[])];
        let tasks = objectives_to_gantt_tasks(&objectives);
        assert!(tasks[0].group.is_none());
    }

    #[test]
    fn test_objectives_to_gantt_tasks_with_no_questions_defaults_to_medium() {
        let objectives = vec![objective("x", vec![], &[])];
        let tasks = objectives_to_gantt_tasks(&objectives);
        assert_eq!(tasks[0].priority, Priority::Medium);
    }
}
