pub mod gantt;
pub mod todo;

use crate::priority::Priority;
use crate::questions::Objective;
use crate::writer::gantt::GanttTask;
use std::collections::HashMap;

/// One objective's worth of questions, ordered for the gantt chart.
#[derive(Debug, Clone)]
pub struct ObjectiveSchedule {
    pub objective: String,
    pub met: bool,
    pub questions: Vec<ScheduledQuestion>,
}

#[derive(Debug, Clone)]
pub struct ScheduledQuestion {
    pub id: usize,
    pub content: String,
    pub priority: Priority,
    pub days: f32,
    pub answered: bool,
    pub prereq: Option<usize>,
    pub actions: Vec<ScheduledAction>,
}

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub content: String,
    pub category: String,
    pub days: f32,
    pub completed: bool,
}

pub fn objectives_to_schedule(objectives: &[Objective]) -> Vec<ObjectiveSchedule> {
    objectives.iter().map(objective_to_schedule).collect()
}

fn objective_to_schedule(o: &Objective) -> ObjectiveSchedule {
    let order = sort_questions_for_gantt(o);
    let qs = o.questions();
    let questions = order
        .into_iter()
        .map(|i| {
            let q = &qs[i];
            ScheduledQuestion {
                id: q.id(),
                content: q.content().to_string(),
                priority: Priority::from(q.priority()),
                days: q.total_time_required(),
                answered: q.answered(),
                prereq: q.prereq(),
                actions: q
                    .actions()
                    .iter()
                    .map(|a| ScheduledAction {
                        content: a.content().to_string(),
                        category: a.category().as_str().to_string(),
                        days: a.required_time(),
                        completed: a.completed(),
                    })
                    .collect(),
            }
        })
        .collect();
    ObjectiveSchedule {
        objective: o.content().to_string(),
        met: o.met(),
        questions,
    }
}

/// Order questions for gantt rendering:
/// - higher priority first;
/// - within the same priority tier, questions with no prereq come before
///   questions with a prereq (singles before chains);
/// - when a question is selected, its full prereq chain is emitted root-first
///   immediately before it, even if some prereqs have lower priority.
fn sort_questions_for_gantt(o: &Objective) -> Vec<usize> {
    let qs = o.questions();
    let n = qs.len();
    let mut placed = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    let priority_score = |i: usize| Priority::from(qs[i].priority()).score();
    let prereq_idx = |i: usize| {
        qs[i]
            .prereq()
            .and_then(|pid| qs.iter().position(|q| q.id() == pid))
    };

    while let Some(seed) = (0..n).filter(|&i| !placed[i]).min_by(|&a, &b| {
        priority_score(b)
            .cmp(&priority_score(a))
            .then(qs[a].prereq().is_some().cmp(&qs[b].prereq().is_some()))
            .then(qs[a].id().cmp(&qs[b].id()))
    }) {
        let mut chain: Vec<usize> = Vec::new();
        let mut cur = Some(seed);
        while let Some(idx) = cur {
            if placed[idx] || chain.contains(&idx) {
                break;
            }
            chain.push(idx);
            cur = prereq_idx(idx);
        }
        chain.reverse();
        for idx in chain {
            order.push(idx);
            placed[idx] = true;
        }
    }
    order
}

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
