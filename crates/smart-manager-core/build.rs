use std::fs;
use std::path::Path;

const TODO_TEMPLATE: &str = "templates/todo.html";
const BODY_MARKER: &str = "<!-- BODY -->";

fn main() {
    println!("cargo:rerun-if-changed={}", TODO_TEMPLATE);
    println!("cargo:rerun-if-changed=build.rs");

    let template = fs::read_to_string(TODO_TEMPLATE)
        .unwrap_or_else(|e| panic!("read template '{}': {}", TODO_TEMPLATE, e));

    let (head, tail) = template.split_once(BODY_MARKER).unwrap_or_else(|| {
        panic!(
            "template '{}' must contain exactly one '{}' marker",
            TODO_TEMPLATE, BODY_MARKER
        )
    });

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let dest = Path::new(&out_dir).join("todo_template.rs");
    let generated = format!(
        "const TODO_HTML_HEAD: &str = {:?};\nconst TODO_HTML_TAIL: &str = {:?};\n",
        head, tail
    );
    fs::write(&dest, generated).expect("write generated todo_template.rs");
}
