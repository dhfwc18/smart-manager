use crate::models::{ActionCategory, ActionPoint, Objective, Question, QuestionPriority, Tag};

pub fn objectives() -> Vec<Objective> {
    let mut ship = Objective::new("Ship MVP".into());
    ship.add_tag(Tag::new("product"));
    ship.add_tag(Tag::new("Q2"));

    let mut q_model = Question::new("Is the data model right?".into(), QuestionPriority::High);
    let mut sketch = ActionPoint::new("Sketch schema".into(), ActionCategory::Writing, 1.0);
    sketch.set_completed(true);
    q_model.push_action(sketch);
    q_model.push_action(ActionPoint::new(
        "Review with team".into(),
        ActionCategory::Managing,
        2.0,
    ));
    ship.push_question(q_model);

    let mut q_platform = Question::new("Which platforms first?".into(), QuestionPriority::Medium);
    q_platform.push_action(ActionPoint::new(
        "Pick desktop target".into(),
        ActionCategory::Research,
        1.0,
    ));
    q_platform.push_action(ActionPoint::new(
        "Spike Dioxus desktop build".into(),
        ActionCategory::Programming,
        3.0,
    ));
    ship.push_question(q_platform);

    let mut pilot = Objective::new("Onboard pilot user".into());
    pilot.add_tag(Tag::new("growth"));
    let mut q_pilot = Question::new("Who is the pilot?".into(), QuestionPriority::Critical);
    q_pilot.push_action(ActionPoint::new(
        "Draft outreach list".into(),
        ActionCategory::Writing,
        0.5,
    ));
    pilot.push_question(q_pilot);

    vec![ship, pilot]
}
