use smart_manager_core::core::App;
use smart_manager_core::questions::{
    ActionCategory, ActionPoint, Objective, Question, QuestionPriority, Tag,
};
use smart_manager_core::writer::gantt::{self, GanttFormat};
use smart_manager_core::writer::objectives_to_gantt_tasks;
use smart_manager_core::writer::todo::{self, TodoFormat};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    let (mut app, autosave) = handle_args();
    repl(&mut app, autosave.as_deref());
}

fn handle_args() -> (App, Option<PathBuf>) {
    let args: Vec<String> = std::env::args().collect();
    let mut app = App::new();
    let mut autosave: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--load" => {
                let path = match args.get(i + 1) {
                    Some(p) => p,
                    None => {
                        eprintln!("--load requires a path argument");
                        std::process::exit(2);
                    }
                };
                match App::load(Path::new(path)) {
                    Ok(loaded) => {
                        app = loaded;
                        println!("loaded {path}");
                    }
                    Err(e) => eprintln!("load failed: {e}"),
                }
                i += 2;
            }
            "--autosave" => {
                let path = match args.get(i + 1) {
                    Some(p) => p,
                    None => {
                        eprintln!("--autosave requires a path argument");
                        std::process::exit(2);
                    }
                };
                autosave = Some(PathBuf::from(path));
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    (app, autosave)
}

fn print_help() {
    println!("smanager - smart manager CLI");
    println!();
    println!("USAGE: smanager [--load <path>] [--autosave <path>]");
    println!();
    println!("Commands at the prompt:");
    println!("  /help                  show this help");
    println!("  /quit | /exit          exit");
    println!("  /list                  list objectives, questions, actions");
    println!("  /set objective <text>  add objective; then prompts for questions");
    println!("                         and actions until empty input");
    println!("  /load <path>           load app state from JSON");
    println!("  /save <path>           save app state to JSON");
    println!("  /dashboard             TODO list + ASCII gantt");
    println!("  /complete <i.j.k>      mark action complete");
    println!("  /answer <i.j>          mark question answered");
    println!("  /met <i>               mark objective met");
    println!("  /tag <i> <name>        add tag to objective");
}

fn repl(app: &mut App, autosave: Option<&Path>) {
    println!("smanager - type /help for commands");
    if let Some(p) = autosave {
        println!("autosave -> {}", p.display());
    }
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("input error: {e}");
                break;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !run(app, line) {
            break;
        }
        if let Some(path) = autosave {
            if is_mutating(line) {
                if let Err(e) = app.save(path) {
                    eprintln!("autosave failed: {e}");
                }
            }
        }
    }
}

fn is_mutating(line: &str) -> bool {
    let cmd = line.split_once(' ').map(|(c, _)| c).unwrap_or(line);
    matches!(
        cmd,
        "/set" | "/load" | "/complete" | "/answer" | "/met" | "/tag"
    )
}

fn run(app: &mut App, line: &str) -> bool {
    let (cmd, rest) = match line.split_once(' ') {
        Some((c, r)) => (c, r.trim()),
        None => (line, ""),
    };
    match cmd {
        "/help" => print_help(),
        "/quit" | "/exit" => return false,
        "/list" => cmd_list(app),
        "/set" => cmd_set(app, rest),
        "/load" => cmd_load(app, rest),
        "/save" => cmd_save(app, rest),
        "/dashboard" => cmd_dashboard(app),
        "/complete" => cmd_complete(app, rest),
        "/answer" => cmd_answer(app, rest),
        "/met" => cmd_met(app, rest),
        "/tag" => cmd_tag(app, rest),
        other => println!("unknown command: {other} (try /help)"),
    }
    true
}

fn priority_label(p: &QuestionPriority) -> &'static str {
    match p {
        QuestionPriority::Critical => "Critical",
        QuestionPriority::High => "High",
        QuestionPriority::Medium => "Medium",
        QuestionPriority::Low => "Low",
        QuestionPriority::LongTerm => "LongTerm",
    }
}

fn parse_priority(s: &str) -> Option<QuestionPriority> {
    match s.trim().to_lowercase().as_str() {
        "critical" | "c" => Some(QuestionPriority::Critical),
        "high" | "h" => Some(QuestionPriority::High),
        "" | "medium" | "m" => Some(QuestionPriority::Medium),
        "low" | "l" => Some(QuestionPriority::Low),
        "longterm" | "long" | "lt" => Some(QuestionPriority::LongTerm),
        _ => None,
    }
}

fn parse_category(s: &str) -> Option<ActionCategory> {
    match s.trim().to_lowercase().as_str() {
        "" | "writing" | "w" => Some(ActionCategory::Writing),
        "managing" | "mng" => Some(ActionCategory::Managing),
        "qa" | "q" => Some(ActionCategory::Qa),
        "analysis" | "a" => Some(ActionCategory::Analysis),
        "research" | "r" => Some(ActionCategory::Research),
        "programming" | "prog" | "code" => Some(ActionCategory::Programming),
        "presentation" | "pres" => Some(ActionCategory::Presentation),
        _ => None,
    }
}

fn prompt(text: &str) -> Option<String> {
    print!("{text}");
    io::stdout().flush().ok();
    let mut s = String::new();
    let n = io::stdin().read_line(&mut s).ok()?;
    if n == 0 {
        return None;
    }
    Some(s.trim().to_string())
}

fn cmd_list(app: &App) {
    if app.objectives().is_empty() {
        println!("(no objectives)");
        return;
    }
    for (oi, o) in app.objectives().iter().enumerate() {
        let met = if o.met() { "[x]" } else { "[ ]" };
        let tags = o
            .tags()
            .iter()
            .map(|t| t.name())
            .collect::<Vec<_>>()
            .join(",");
        let tag_str = if tags.is_empty() {
            String::new()
        } else {
            format!(" #{tags}")
        };
        println!("{met} [{oi}] {}{tag_str}", o.content());
        for (qi, q) in o.questions().iter().enumerate() {
            let ans = if q.answered() { "[x]" } else { "[ ]" };
            println!(
                "  {ans} [{oi}.{qi}] {} ({})",
                q.content(),
                priority_label(q.priority())
            );
            for (ai, a) in q.actions().iter().enumerate() {
                let comp = if a.completed() { "[x]" } else { "[ ]" };
                println!(
                    "    {comp} [{oi}.{qi}.{ai}] {} ({}d, {})",
                    a.content(),
                    a.required_time(),
                    a.category().as_str()
                );
            }
        }
    }
}

fn cmd_set(app: &mut App, args: &str) {
    let (sub, text) = match args.split_once(' ') {
        Some((s, t)) => (s, t.trim()),
        None => (args, ""),
    };
    if sub != "objective" {
        println!("usage: /set objective <text>");
        return;
    }
    if text.is_empty() {
        println!("usage: /set objective <text>");
        return;
    }
    let oi = app.objectives().len();
    let mut o = Objective::new(text.to_string());
    println!("adding objective [{oi}] {text}");
    println!("add questions (empty line to finish):");
    let mut qi = 0;
    loop {
        let q_text = match prompt(&format!("  q[{oi}.{qi}]> ")) {
            Some(s) if !s.is_empty() => s,
            _ => break,
        };
        let p = prompt("    priority [Critical/High/Medium/Low/LongTerm] (Medium): ")
            .unwrap_or_default();
        let priority = parse_priority(&p).unwrap_or(QuestionPriority::Medium);
        let mut q = Question::new(q_text, priority);
        let mut ai = 0;
        loop {
            let a_text = match prompt(&format!("    a[{oi}.{qi}.{ai}]> ")) {
                Some(s) if !s.is_empty() => s,
                _ => break,
            };
            let c = prompt("      category [Writing/Managing/Qa/Analysis/Research/Programming/Presentation] (Writing): ")
                .unwrap_or_default();
            let category = parse_category(&c).unwrap_or(ActionCategory::Writing);
            let t = prompt("      required time in days (1.0): ").unwrap_or_default();
            let days: f32 = t.parse().unwrap_or(1.0);
            q.push_action(ActionPoint::new(a_text, category, days));
            ai += 1;
        }
        o.push_question(q);
        qi += 1;
    }
    app.push_objective(o);
    println!("added [{oi}]");
}

fn cmd_load(app: &mut App, path: &str) {
    if path.is_empty() {
        println!("usage: /load <path>");
        return;
    }
    match App::load(Path::new(path)) {
        Ok(loaded) => {
            *app = loaded;
            println!("loaded {path}");
        }
        Err(e) => println!("load failed: {e}"),
    }
}

fn cmd_save(app: &App, path: &str) {
    if path.is_empty() {
        println!("usage: /save <path>");
        return;
    }
    match app.save(Path::new(path)) {
        Ok(()) => println!("saved {path}"),
        Err(e) => println!("save failed: {e}"),
    }
}

fn cmd_dashboard(app: &App) {
    let tasks = objectives_to_gantt_tasks(app.objectives());
    println!("===== TODOs =====\n");
    println!("{}", todo::render(&tasks, TodoFormat::Markdown));
    println!("===== Gantt =====\n");
    println!("{}", gantt::render(&tasks, GanttFormat::Ascii));
}

fn parse_id(s: &str) -> Option<Vec<usize>> {
    if s.is_empty() {
        return None;
    }
    s.split('.').map(|p| p.parse().ok()).collect()
}

fn cmd_complete(app: &mut App, id: &str) {
    let parts = match parse_id(id) {
        Some(p) if p.len() == 3 => p,
        _ => {
            println!("usage: /complete <obj.question.action>");
            return;
        }
    };
    let (oi, qi, ai) = (parts[0], parts[1], parts[2]);
    let action = app
        .objective_mut(oi)
        .and_then(|o| o.question_mut(qi))
        .and_then(|q| q.action_mut(ai));
    match action {
        Some(a) => {
            a.set_completed(true);
            println!("marked [{oi}.{qi}.{ai}] complete");
        }
        None => println!("not found: [{oi}.{qi}.{ai}]"),
    }
}

fn cmd_answer(app: &mut App, id: &str) {
    let parts = match parse_id(id) {
        Some(p) if p.len() == 2 => p,
        _ => {
            println!("usage: /answer <obj.question>");
            return;
        }
    };
    let (oi, qi) = (parts[0], parts[1]);
    let q = app.objective_mut(oi).and_then(|o| o.question_mut(qi));
    match q {
        Some(q) => match q.set_answered(true) {
            Ok(()) => println!("marked [{oi}.{qi}] answered"),
            Err(e) => println!("cannot answer: {e}"),
        },
        None => println!("not found: [{oi}.{qi}]"),
    }
}

fn cmd_met(app: &mut App, id: &str) {
    let parts = match parse_id(id) {
        Some(p) if p.len() == 1 => p,
        _ => {
            println!("usage: /met <obj>");
            return;
        }
    };
    let oi = parts[0];
    match app.objective_mut(oi) {
        Some(o) => match o.set_met(true) {
            Ok(()) => println!("marked [{oi}] met"),
            Err(e) => println!("cannot mark met: {e}"),
        },
        None => println!("not found: [{oi}]"),
    }
}

fn cmd_tag(app: &mut App, args: &str) {
    let parts: Vec<&str> = args.splitn(2, ' ').collect();
    if parts.len() != 2 {
        println!("usage: /tag <obj> <name>");
        return;
    }
    let oi: usize = match parts[0].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("invalid index: {}", parts[0]);
            return;
        }
    };
    let name = parts[1].trim();
    if name.is_empty() {
        println!("tag name required");
        return;
    }
    match app.add_tag(oi, Tag::new(name)) {
        Ok(true) => println!("tagged [{oi}] with {name}"),
        Ok(false) => println!("[{oi}] already has tag {name}"),
        Err(_) => println!("not found: [{oi}]"),
    }
}
