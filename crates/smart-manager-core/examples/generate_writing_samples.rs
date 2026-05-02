//! Renders writer artifacts in `samples/` from `samples/case.json`.
//!
//! `case.json` is the committed source of truth for the demo App state.
//! Edit it by hand (or load it into the CLI, mutate, save) — this binary
//! only reads it and emits the rendered outputs alongside.
//!
//! Run with:
//!     cargo run -p smart-manager-core --example generate_writing_samples
//!
//! Output is overwritten in place. Skippable by default — this binary is not
//! part of `cargo build` or `cargo test`.

use smart_manager_core::core::App;
use smart_manager_core::writer::gantt::GanttFormat;
use smart_manager_core::writer::todo::TodoFormat;
use std::fs;
use std::path::{Path, PathBuf};

const CASE_FILE: &str = "case.json";

fn main() {
    let dir = samples_dir();
    let case = dir.join(CASE_FILE);

    let app = App::load(&case).unwrap_or_else(|e| panic!("load {}: {e}", case.display()));

    write(&dir, "gantt.md", &app.render_gantt(GanttFormat::Markdown));
    write(&dir, "gantt.txt", &app.render_gantt(GanttFormat::Ascii));
    write(&dir, "gantt.json", &app.render_gantt(GanttFormat::Json));
    write(&dir, "todo.md", &app.render_todo(TodoFormat::Markdown));
    write(&dir, "todo.html", &app.render_todo(TodoFormat::Html));

    println!("rendered {} artifacts from {}", 5, case.display());
}

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples")
}

fn write(dir: &Path, name: &str, contents: &str) {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    println!("  {name}");
}
