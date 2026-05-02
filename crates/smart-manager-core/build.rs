use std::fs;
use std::path::Path;

const BODY_MARKER: &str = "<!-- BODY -->";

/// Templates that wrap dynamic body content. Each template file must contain
/// exactly one `<!-- BODY -->` marker; the build splits it into HEAD/TAIL
/// constants named `<NAME>_HEAD` and `<NAME>_TAIL`.
const TEMPLATES: &[Template] = &[
    Template {
        path: "templates/todo.html",
        name: "TODO_HTML",
    },
    Template {
        path: "templates/TODO.md",
        name: "TODO_MD",
    },
    Template {
        path: "templates/gantt_chart.md",
        name: "GANTT_CHART_MD",
    },
    Template {
        path: "templates/gantt_table.md",
        name: "GANTT_TABLE_MD",
    },
];

struct Template {
    path: &'static str,
    name: &'static str,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");

    for tpl in TEMPLATES {
        println!("cargo:rerun-if-changed={}", tpl.path);

        let source = fs::read_to_string(tpl.path)
            .unwrap_or_else(|e| panic!("read template '{}': {}", tpl.path, e));
        let (head, tail) = source.split_once(BODY_MARKER).unwrap_or_else(|| {
            panic!(
                "template '{}' must contain exactly one '{}' marker",
                tpl.path, BODY_MARKER
            )
        });

        let dest = Path::new(&out_dir).join(format!("{}_template.rs", tpl.name.to_lowercase()));
        let generated = format!(
            "const {name}_HEAD: &str = {head:?};\nconst {name}_TAIL: &str = {tail:?};\n",
            name = tpl.name,
            head = head,
            tail = tail,
        );
        fs::write(&dest, generated).unwrap_or_else(|e| panic!("write {}: {}", dest.display(), e));
    }
}
