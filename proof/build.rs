use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=formulas/");
    println!("cargo:rerun-if-changed=entities/");
    println!("cargo:rerun-if-changed=../athena/formulas/");
    println!("cargo:rerun-if-changed=../athena/entities/");

    let embedded = out_dir.join("embedded.rs");
    let mut src = String::new();

    // Seed corpus is always embedded so the binary is self-contained regardless
    // of CWD (fixes T35: entities/formulas/forms/events must load from anywhere).

    let formula_files = [
        "formulas/atomic_seed.toml",
        "formulas/atomic_dynamic.toml",
        "formulas/bridging_seed.toml",
        "formulas/vortex_seed.toml",
    ];

    let mut formulas = String::new();
    for rel in formula_files {
        let path = manifest.join(rel);
        if let Ok(content) = std::fs::read_to_string(&path) {
            formulas.push_str(&content);
            formulas.push('\n');
        }
    }

    let synonyms_path = manifest.join("formulas/search_synonyms.toml");
    let synonyms = std::fs::read_to_string(&synonyms_path).unwrap_or_default();

    let nonmath_files = ["formulas/nonmath_seed.toml"];
    let mut nonmath = String::new();
    for rel in nonmath_files {
        let path = manifest.join(rel);
        if let Ok(content) = std::fs::read_to_string(&path) {
            nonmath.push_str(&content);
            nonmath.push('\n');
        }
    }

    let shikai_form_path = manifest.join("shikai_form.toml");
    let shikai_form = std::fs::read_to_string(&shikai_form_path).unwrap_or_default();

    let events_path = manifest.join("events.toml");
    let events_toml = std::fs::read_to_string(&events_path).unwrap_or_default();

    let entities_dir = manifest.join("entities");
    let mut entities = String::new();
    if let Ok(entries) = std::fs::read_dir(&entities_dir) {
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                entities.push_str(&content);
                entities.push('\n');
            }
        }
    }

    src.push_str(&format!(
        "pub const FORMULAS_TOML: &str = {:?};\n",
        formulas
    ));
    src.push_str(&format!(
        "pub const SYNONYMS_TOML: &str = {:?};\n",
        synonyms
    ));
    src.push_str(&format!(
        "pub const ENTITIES_TOML: &str = {:?};\n",
        entities
    ));
    src.push_str(&format!("pub const NONMATH_TOML: &str = {:?};\n", nonmath));
    src.push_str(&format!(
        "pub const SHIKAI_FORM_TOML: &str = {:?};\n",
        shikai_form
    ));
    src.push_str(&format!(
        "pub const EVENTS_TOML: &str = {:?};\n",
        events_toml
    ));

    // ─── Athena corpus (embedded from ../athena/) ──────────────────────────
    let athena_root = manifest.parent().unwrap().join("athena");

    // Athena formulas: atomic + bridging + nonmath
    let athena_formula_dirs = ["formulas/atomic", "formulas/bridging", "formulas/nonmath"];
    let mut athena_formulas = String::new();
    for dir in &athena_formula_dirs {
        let dir_path = athena_root.join(dir);
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "toml"))
                .collect();
            paths.sort();
            for path in paths {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    athena_formulas.push_str(&content);
                    athena_formulas.push('\n');
                }
            }
        }
    }

    // Athena entities
    let athena_entities_dir = athena_root.join("entities");
    let mut athena_entities = String::new();
    if let Ok(entries) = std::fs::read_dir(&athena_entities_dir) {
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                athena_entities.push_str(&content);
                athena_entities.push('\n');
            }
        }
    }

    src.push_str(&format!(
        "pub const ATHENA_FORMULAS_TOML: &str = {:?};\n",
        athena_formulas
    ));
    src.push_str(&format!(
        "pub const ATHENA_ENTITIES_TOML: &str = {:?};\n",
        athena_entities
    ));

    // Versioned corpus manifest: semver (from Cargo.toml) + a stable content
    // hash over the embedded corpus, so downstream tooling / community forks
    // can detect corpus drift. Uses FNV-1a (no external crypto deps, which
    // were deliberately removed) — deterministic across builds and platforms.
    let version = env!("CARGO_PKG_VERSION");
    let mut hasher = Fnv1a64::new();
    hasher.write(formulas.as_bytes());
    hasher.write(nonmath.as_bytes());
    hasher.write(entities.as_bytes());
    let content_hash = hasher.hex();
    let manifest = format!(
        "version = \"{}\"\ncontent_hash = \"{}\"\n",
        version, content_hash
    );
    src.push_str(&format!(
        "pub const CORPUS_VERSION: &str = {:?};\n",
        version
    ));
    src.push_str(&format!(
        "pub const CORPUS_CONTENT_HASH: &str = {:?};\n",
        content_hash
    ));
    src.push_str(&format!(
        "pub const CORPUS_VERSION_TOML: &str = {:?};\n",
        manifest
    ));

    std::fs::write(&embedded, src).expect("write embedded.rs");
}

/// FNV-1a 64-bit hasher — dependency-free, stable across platforms/builds.
struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    fn new() -> Self {
        Fnv1a64 {
            state: 0xcbf29ce484222325,
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        const PRIME: u64 = 0x100000001b3;
        for &b in bytes {
            self.state ^= b as u64;
            self.state = self.state.wrapping_mul(PRIME);
        }
    }

    fn hex(&self) -> String {
        format!("{:016x}", self.state)
    }
}
