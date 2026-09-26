//! JS: lib/staleness.mjs (Tier 1)

use crate::artifact_schema::*;
use crate::context::{BriefSummary, Ctx, TargetCandidate};
use crate::jsp;
use crate::util::{exists, js_trim, mtime_ms, read_json, safe_read};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Finding {
    pub id: String,
    pub artifact: String,
    pub path: Option<String>,
    pub severity: &'static str,
    pub summary: String,
    pub fix: String,
}

impl Finding {
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), Value::String(self.id.clone()));
        m.insert("artifact".into(), Value::String(self.artifact.clone()));
        m.insert("path".into(), self.path.clone().map(Value::String).unwrap_or(Value::Null));
        m.insert("severity".into(), Value::String(self.severity.to_string()));
        m.insert("summary".into(), Value::String(self.summary.clone()));
        m.insert("fix".into(), Value::String(self.fix.clone()));
        Value::Object(m)
    }
}

pub fn finding(id: &str, artifact: &str, path: Option<String>, severity: &'static str, summary: String, fix: String) -> Finding {
    Finding { id: id.to_string(), artifact: artifact.to_string(), path, severity, summary, fix }
}

const KNOWN_CONFIG_KEYS: [&str; 9] =
    ["hook", "detector", "updateCheck", "stalenessCheck", "projectRoots", "buildPath", "browser", "$schema", "version"];
const BUILD_PATH_VALUES: [&str; 2] = ["comp", "code"];
const DIRECTION_WORK_PATHS: [&str; 2] = [".impeccable/surfaces", ".impeccable/mocks/decision"];
const KNOWN_DETECTOR_KEYS: [&str; 5] = ["ignoreRules", "ignoreFiles", "ignoreValues", "designSystem", "extensions"];

struct NativeEvidence {
    platform: &'static str,
    reason: &'static str,
}
const NATIVE_EVIDENCE_PATHS: [(&str, &str, &str); 5] = [
    ("pubspec.yaml", "adaptive", "a Flutter pubspec.yaml"),
    ("ios/Podfile", "ios", "an ios/Podfile"),
    ("android/build.gradle", "android", "an android/build.gradle"),
    ("android/build.gradle.kts", "android", "an android/build.gradle.kts"),
    ("ios/Runner.xcodeproj", "ios", "an ios/Runner.xcodeproj"),
];
const NATIVE_EVIDENCE_DEPENDENCIES: [(&str, &str, &str); 4] = [
    ("react-native", "adaptive", "a react-native dependency"),
    ("expo", "adaptive", "an expo dependency"),
    ("@react-native/metro-config", "adaptive", "a React Native metro config dependency"),
    ("ink", "terminal", "an ink dependency"),
];
/// Text manifests at the project root whose dependency entries name a
/// terminal stack: (manifest file, [(dependency, reason)]). Each manifest is
/// read once and parsed by `manifest_names_dependency`. Only the root
/// manifest is read (Tier 1 walks nothing), so Cargo member crates are not
/// seen. `rich` and `crossterm` are absent on purpose: web backends and plain
/// CLIs use both without a full-screen UI, so neither is evidence alone.
const TERMINAL_EVIDENCE_MANIFESTS: [(&str, &[(&str, &str)]); 4] = [
    ("Cargo.toml", &[("ratatui", "a ratatui dependency")]),
    (
        "go.mod",
        &[
            ("bubbletea", "a bubbletea dependency"),
            ("lipgloss", "a lipgloss dependency"),
            ("bubbles", "a bubbles dependency"),
        ],
    ),
    ("pyproject.toml", &[("textual", "a textual dependency")]),
    ("requirements.txt", &[("textual", "a textual dependency")]),
];

/// True when `manifest`'s text names the dependency `name`.
fn manifest_names_dependency(manifest: &str, text: &str, name: &str) -> bool {
    match manifest {
        "go.mod" => go_mod_requires(text, name),
        "Cargo.toml" => cargo_names_dependency(text, name),
        _ => python_names_dependency(text, name),
    }
}

/// Charm modules live under both hosts: v1 under GitHub, v2 under the
/// `charm.land` vanity path (`charm.land/bubbletea/v2`).
const CHARM_MODULE_HOSTS: [&str; 2] = ["github.com/charmbracelet/", "charm.land/"];

/// go.mod: true when a `require` entry names the Charm module `name`. A
/// single-line `require <path> <version>` and the lines of a `require (` block
/// count. Nothing inside another block (`replace (`, `exclude (`, `retract (`,
/// `tool (`, `godebug (`) counts, and an `// indirect` entry does not.
fn go_mod_requires(text: &str, name: &str) -> bool {
    // None outside a block; Some(true) inside `require (`; Some(false) inside any other block.
    let mut block: Option<bool> = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(in_require) = block {
            if t.starts_with(')') {
                block = None;
            } else if in_require && go_entry_names(t, name) {
                return true;
            }
            continue;
        }
        let verb_end = t.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(t.len());
        let (verb, rest) = (&t[..verb_end], t[verb_end..].trim_start());
        if rest.starts_with('(') {
            block = Some(verb == "require");
        } else if verb == "require" && go_entry_names(rest, name) {
            return true;
        }
    }
    false
}

/// One require entry, `<module path> <version> [// comment]`: it names `name`
/// when the path is `<host><name>` or `<host><name>/v<digits>` for a Charm
/// host and the comment does not mark it indirect (`indirect`, or
/// `indirect;` followed by more, as `golang.org/x/mod/modfile` reads it).
fn go_entry_names(entry: &str, name: &str) -> bool {
    if let Some(i) = entry.find("//") {
        let comment = entry[i + 2..].trim();
        if comment == "indirect" || comment.starts_with("indirect;") {
            return false;
        }
    }
    let path = entry.split_whitespace().next().unwrap_or("").trim_matches('"');
    CHARM_MODULE_HOSTS.iter().any(|host| match path.strip_prefix(host).and_then(|p| p.strip_prefix(name)) {
        Some("") => true,
        Some(rest) => rest
            .strip_prefix("/v")
            .map_or(false, |digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())),
        None => false,
    })
}

/// pyproject.toml and requirements.txt: after stripping a `#` comment, the
/// line itself (a requirements line, or a Poetry `textual = "^0.80"` key) and
/// every quoted string on it (a PEP 621 array such as
/// `dependencies = ["Textual>=0.80", "rich"]`) are read as requirements.
fn python_names_dependency(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let line = line.split('#').next().unwrap_or("");
        python_requirement_names(line, name)
            || line.split(['"', '\'']).skip(1).step_by(2).any(|s| python_requirement_names(s, name))
    })
}

/// A requirement names the package `name` (already normalized) when its
/// leading name run, PEP 503 normalized, equals it and what follows is a
/// requirement tail: nothing, extras `[`, a version `( < > = ! ~`, a marker
/// `;`, or a URL `@`. `=` also admits a Poetry key. Prose such as
/// "Textual app for notes" fails the tail check. Accepted residuals:
/// `keywords = ["textual"]` and a project named `textual` still match.
fn python_requirement_names(spec: &str, name: &str) -> bool {
    let spec = spec.trim_start();
    if !spec.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return false;
    }
    let end = spec
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        .unwrap_or(spec.len());
    if pep503_normalize(&spec[..end]) != name {
        return false;
    }
    let tail = spec[end..].trim_start();
    tail.is_empty() || tail.starts_with(['[', '(', '<', '>', '=', '!', '~', ';', '@'])
}

/// PEP 503 name normalization: lowercase, each run of `-`, `_`, `.` becomes
/// one `-`.
fn pep503_normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

/// Cargo.toml: a key line naming the crate (`ratatui = "0.29"`,
/// `ratatui.workspace = true`, `"ratatui" = ...`) or a table header ending in
/// `dependencies.ratatui]` (`[dependencies.ratatui]`,
/// `[target.'cfg(unix)'.dependencies.ratatui]`). `#` comments are ignored.
fn cargo_names_dependency(text: &str, name: &str) -> bool {
    let header_tail = format!("dependencies.{}]", name);
    text.lines().any(|line| {
        let t = line.split('#').next().unwrap_or("").trim();
        if t.starts_with('[') {
            return t.ends_with(&header_tail);
        }
        let key = t.strip_prefix('"').unwrap_or(t);
        match key.strip_prefix(name) {
            Some(rest) => {
                let rest = rest.strip_prefix('"').unwrap_or(rest);
                rest.starts_with(|c: char| c.is_whitespace() || c == '=' || c == '.')
            }
            None => false,
        }
    })
}

/// JS: designSidecarCandidatesFor(projectRoot, contextDir)
pub fn design_sidecar_candidates_for(project_root: &str, context_dir: Option<&str>) -> Vec<String> {
    let mut c = vec![jsp::join(&[project_root, ".impeccable", "design.json"]), jsp::join(&[project_root, "DESIGN.json"])];
    let ctx_legacy = jsp::join(&[context_dir.unwrap_or(project_root), "DESIGN.json"]);
    if !c.contains(&ctx_legacy) {
        c.push(ctx_legacy);
    }
    c
}

fn has_section(markdown: &str, heading: &str) -> bool {
    Regex::new(&format!(r"(?im)^##\s+{}\s*$", regex::escape(heading))).map(|r| r.is_match(markdown)).unwrap_or(false)
}

pub fn to_relative(file_path: Option<&str>, root: &str) -> Option<String> {
    let fp = file_path?;
    let rel = jsp::relative("/", root, fp);
    if !rel.is_empty() && !rel.starts_with("..") && !jsp::is_absolute(&rel) {
        Some(jsp::to_posix(&rel))
    } else {
        Some(fp.to_string())
    }
}

fn wrap_ticks(items: &[String]) -> String {
    items.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", ")
}

/// JS: checkProduct
pub fn check_product(product: Option<&str>, product_path: &str) -> Vec<Finding> {
    let Some(product) = product.filter(|p| !p.is_empty()) else { return vec![] };
    let mut out = Vec::new();
    for (heading, reason) in PRODUCT_DEPRECATED_SECTIONS {
        if !has_section(product, heading) {
            continue;
        }
        out.push(finding(
            &format!("product-deprecated-{}", heading.to_lowercase()),
            "PRODUCT.md",
            Some(product_path.to_string()),
            "mention",
            format!("PRODUCT.md still carries a `## {}` section. {}", heading, reason),
            format!(
                "Treat `## {}` as absent for every decision this session. Offer to delete the section; do not let its value influence the work either way.",
                heading
            ),
        ));
    }
    let stamped = read_product_schema_version(product);
    if stamped.is_none() && !PRODUCT_V4_SECTIONS.iter().any(|s| has_section(product, s)) {
        out.push(finding(
            "product-schema-legacy",
            "PRODUCT.md",
            Some(product_path.to_string()),
            "route",
            format!(
                "PRODUCT.md has no schema stamp and none of the sections the current record adds ({}), so it predates this version of the product record.",
                PRODUCT_V4_SECTIONS.join(", ")
            ),
            "Offer `init`, which preserves confirmed answers and fills the gaps by interview. Do not rewrite the file from inference.".to_string(),
        ));
    } else if let Some(v) = stamped {
        if v < PRODUCT_SCHEMA_VERSION {
            out.push(finding(
                "product-schema-outdated",
                "PRODUCT.md",
                Some(product_path.to_string()),
                "route",
                format!("PRODUCT.md is stamped product-schema {}; the current record is {}.", v, PRODUCT_SCHEMA_VERSION),
                "Offer `init` to bring the record current, preserving confirmed answers.".to_string(),
            ));
        }
    }
    out
}

/// JS: checkNativePlatformEvidence
pub fn check_native_platform_evidence(
    project_root: &str,
    platform: Option<&str>,
    product: Option<&str>,
    product_path: Option<&str>,
) -> Vec<Finding> {
    if project_root.is_empty() {
        return vec![];
    }
    if let Some(p) = platform {
        if !p.is_empty() && p != "web" {
            return vec![];
        }
    }
    let mut evidence: Vec<NativeEvidence> = Vec::new();
    for (rel, platform, reason) in NATIVE_EVIDENCE_PATHS {
        if exists(&jsp::join(&[project_root, rel])) {
            evidence.push(NativeEvidence { platform, reason });
        }
    }
    if let Some(pkg) = read_json(&jsp::join(&[project_root, "package.json"])) {
        // JS: { ...pkg.dependencies, ...pkg.devDependencies } then deps[name] truthy
        let dep_truthy = |name: &str| -> bool {
            let mut v: Option<&Value> = None;
            if let Some(d) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                if let Some(x) = d.get(name) {
                    v = Some(x);
                }
            }
            if let Some(d) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
                if let Some(x) = d.get(name) {
                    v = Some(x);
                }
            }
            match v {
                None => false,
                Some(x) => js_truthy(x),
            }
        };
        if pkg.is_object() || pkg.is_array() || (!pkg.is_null() && js_truthy(&pkg)) {
            for (name, platform, reason) in NATIVE_EVIDENCE_DEPENDENCIES {
                if dep_truthy(name) {
                    evidence.push(NativeEvidence { platform, reason });
                }
            }
        }
    }
    for (manifest, rows) in TERMINAL_EVIDENCE_MANIFESTS {
        let Some(text) = safe_read(&jsp::join(&[project_root, manifest])) else { continue };
        for &(name, reason) in rows {
            // One reason per dependency even when two manifests (or two
            // module hosts) name it.
            if manifest_names_dependency(manifest, &text, name) && !evidence.iter().any(|e| e.reason == reason) {
                evidence.push(NativeEvidence { platform: "terminal", reason });
            }
        }
    }
    if evidence.is_empty() {
        return vec![];
    }
    let mut platforms: Vec<&str> = Vec::new();
    for e in &evidence {
        if !platforms.contains(&e.platform) {
            platforms.push(e.platform);
        }
    }
    // Mobile evidence outranks terminal evidence: a Flutter app with a Rust
    // helper is still a mobile app. Terminal is suggested only when it is the
    // only kind of evidence present.
    let mobile: Vec<&str> = platforms.iter().copied().filter(|p| *p != "terminal").collect();
    let suggested = if mobile.is_empty() {
        "terminal"
    } else if mobile.len() > 1 || mobile.contains(&"adaptive") {
        "adaptive"
    } else {
        mobile[0]
    };
    let consequence = if suggested == "terminal" {
        "Web guidance is being applied to a terminal codebase, and the terminal reference never loads."
    } else {
        "Web guidance is being applied to a native codebase, and the iOS and Android references never load."
    };
    let reference = if suggested == "terminal" { "terminal" } else { "native" };
    let declared = if platform == Some("web") {
        "PRODUCT.md declares `## Platform: web`"
    } else if product.map(|p| !p.is_empty()).unwrap_or(false) {
        "PRODUCT.md has no `## Platform` section, so the project resolves to web"
    } else {
        "no PRODUCT.md declares a platform, so the project resolves to web"
    };
    vec![finding(
        "platform-native-evidence",
        "PRODUCT.md",
        product_path.map(|s| s.to_string()),
        "mention",
        format!(
            "{}, but the project carries {}. {}",
            declared,
            evidence.iter().map(|e| e.reason).collect::<Vec<_>>().join(" and "),
            consequence
        ),
        format!(
            "Ask the user whether `## Platform` should be `{}`. If it should, write the value and load the matching {} reference before designing.",
            suggested, reference
        ),
    )]
}

pub fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// JS: checkDesignSidecar
pub fn check_design_sidecar(design_path: Option<&str>, sidecar_candidates: &[String], project_root: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let canonical = sidecar_candidates.first();
    let Some(present) = sidecar_candidates.iter().find(|c| exists(c)) else { return out };
    let rel_present = to_relative(Some(present), project_root).unwrap();
    if let Some(canon) = canonical {
        if jsp::resolve(present, &[]) != jsp::resolve(canon, &[]) {
            out.push(finding(
                "design-sidecar-legacy-path",
                "design.json",
                Some(rel_present.clone()),
                "auto",
                format!("The design sidecar sits at {}, a location kept only for backward compatibility.", rel_present),
                format!(
                    "Move it to {} the next time the sidecar is written. No user decision is needed.",
                    to_relative(Some(canon), project_root).unwrap()
                ),
            ));
        }
    }
    let sidecar = read_json(present);
    let schema_version = read_sidecar_schema_version(sidecar.as_ref());
    if let Some(sc) = &sidecar {
        if js_truthy(sc) && (schema_version.is_none() || schema_version.unwrap() < DESIGN_SIDECAR_SCHEMA_VERSION) {
            out.push(finding(
                "design-sidecar-schema-outdated",
                "design.json",
                Some(rel_present.clone()),
                "route",
                format!(
                    "{} is schemaVersion {}; the current sidecar is {}. Token primitives moved to the DESIGN.md frontmatter, so the old shape carries values that are now read from two places.",
                    rel_present,
                    schema_version.map(|v| v.to_string()).unwrap_or_else(|| "unset".to_string()),
                    DESIGN_SIDECAR_SCHEMA_VERSION
                ),
                "Offer `document` to regenerate the sidecar. It reads the existing DESIGN.md, so no interview is needed.".to_string(),
            ));
        }
    }
    if let Some(dp) = design_path {
        let dm = mtime_ms(dp);
        let sm = mtime_ms(present);
        if let (Some(d), Some(s)) = (dm, sm) {
            if d > s {
                out.push(finding(
                    "design-sidecar-stale",
                    "design.json",
                    Some(rel_present.clone()),
                    "mention",
                    format!(
                        "DESIGN.md was edited after {} was generated, so the sidecar's ramps, shadows, motion tokens, and component snippets may contradict it.",
                        rel_present
                    ),
                    "Offer `document` to refresh the sidecar, preserving DESIGN.md.".to_string(),
                ));
            }
        }
    }
    out
}

pub fn unique_roots(a: &str, b: Option<&str>) -> Vec<String> {
    let mut roots = vec![jsp::resolve(a, &[])];
    if let Some(b) = b {
        if !b.is_empty() {
            let r = jsp::resolve(b, &[]);
            if !roots.contains(&r) {
                roots.push(r);
            }
        }
    }
    roots
}

/// JS: checkConfig
pub fn check_config(project_root: &str, repo_root: Option<&str>) -> Vec<Finding> {
    let mut out = Vec::new();
    for root in unique_roots(project_root, repo_root) {
        for name in ["config.json", "config.local.json"] {
            let fp = jsp::join(&[&root, ".impeccable", name]);
            let Some(raw) = read_json(&fp) else { continue };
            let Some(obj) = raw.as_object() else { continue };
            let rel = to_relative(Some(&fp), if project_root.is_empty() { &root } else { project_root }).unwrap();
            let unknown: Vec<String> = obj.keys().filter(|k| !KNOWN_CONFIG_KEYS.contains(&k.as_str())).cloned().collect();
            if !unknown.is_empty() {
                out.push(finding(
                    "config-unknown-keys",
                    "config.json",
                    Some(rel.clone()),
                    "mention",
                    format!(
                        "{} has top-level key(s) nothing reads: {}. Recognized keys are {}.",
                        rel,
                        wrap_ticks(&unknown),
                        wrap_ticks(&KNOWN_CONFIG_KEYS.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                    ),
                    "Report the exact keys to the user. A near-miss of a real key is a setting that has never applied.".to_string(),
                ));
            }
            if let Some(bp) = obj.get("buildPath") {
                let ok = bp.as_str().map(|s| BUILD_PATH_VALUES.contains(&s)).unwrap_or(false);
                if !ok {
                    out.push(finding(
                        "config-invalid-build-path",
                        "config.json",
                        Some(rel.clone()),
                        "mention",
                        format!(
                            "{} sets `buildPath` to {}, which nothing reads. The values are {}.",
                            rel,
                            js_json_stringify(bp),
                            BUILD_PATH_VALUES.iter().map(|v| format!("`{}`", v)).collect::<Vec<_>>().join(" and ")
                        ),
                        "Report the value. An unread `buildPath` does not fall back to the other path; it falls back to the default, so a project meaning `code` has been building comp-led.".to_string(),
                    ));
                }
            }
            if let Some(det) = obj.get("detector").and_then(|d| d.as_object()) {
                let unknown_d: Vec<String> = det.keys().filter(|k| !KNOWN_DETECTOR_KEYS.contains(&k.as_str())).cloned().collect();
                if !unknown_d.is_empty() {
                    out.push(finding(
                        "config-unknown-detector-keys",
                        "config.json",
                        Some(rel.clone()),
                        "mention",
                        format!(
                            "{} has `detector` key(s) nothing reads: {}. Recognized keys are {}.",
                            rel,
                            wrap_ticks(&unknown_d),
                            wrap_ticks(&KNOWN_DETECTOR_KEYS.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                        ),
                        "Report the exact keys. `ignoreRule` for `ignoreRules` is the common one, and it silences nothing.".to_string(),
                    ));
                }
            }
        }
    }
    out
}

/// `JSON.stringify(v)` for a single value (undefined -> "undefined" never occurs here since key present).
pub fn js_json_stringify(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

/// JS: checkBuildPathUnset
pub fn check_build_path_unset(project_root: &str, repo_root: Option<&str>, product: Option<&str>) -> Vec<Finding> {
    if project_root.is_empty() || !product.map(|p| !p.is_empty()).unwrap_or(false) {
        return vec![];
    }
    for root in unique_roots(project_root, repo_root) {
        for name in ["config.json", "config.local.json"] {
            if let Some(raw) = read_json(&jsp::join(&[&root, ".impeccable", name])) {
                if js_truthy(&raw) && raw.as_object().map(|o| o.contains_key("buildPath")).unwrap_or(false) {
                    return vec![];
                }
            }
        }
    }
    let evidence: Vec<&str> = DIRECTION_WORK_PATHS.iter().copied().filter(|rel| exists(&jsp::join(&[project_root, rel]))).collect();
    if evidence.is_empty() {
        return vec![];
    }
    vec![finding(
        "config-build-path-unset",
        "config.json",
        Some(".impeccable/config.json".to_string()),
        "mention",
        "This project has run visual direction work but records no `buildPath`, so every direction round takes the comp-first default without anyone having chosen it.".to_string(),
        "Only when image generation exists in your tool surface, offer the choice once: **comp-first** (an image sets the bar before any code; bolder composition, slower) or **code-first** (build directly; ambition carried by the direction contract; leaner, faster). Write the answer to `.impeccable/config.json` as `\"buildPath\": \"comp\"` or `\"buildPath\": \"code\"`, merging with the keys already there. Without image generation there is no choice to record: stay silent.".to_string(),
    )]
}

/// JS: checkSurfaceBriefs
pub fn check_surface_briefs(candidates: &[BriefSummary], project_root: &str) -> Vec<Finding> {
    if project_root.is_empty() {
        return vec![];
    }
    let mut orphaned: Vec<&BriefSummary> = Vec::new();
    for b in candidates {
        let Some(t) = b.primary_target.as_deref() else { continue };
        if t.is_empty() {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://") || t.starts_with("route:") {
            continue;
        }
        if !exists(&jsp::join(&[project_root, t])) {
            orphaned.push(b);
        }
    }
    if orphaned.is_empty() {
        return vec![];
    }
    let paths: Vec<String> = orphaned.iter().map(|b| b.path.clone()).filter(|p| !p.is_empty()).collect();
    vec![finding(
        "surface-brief-orphaned",
        "surface brief",
        if paths.is_empty() { None } else { Some(paths.join(", ")) },
        "mention",
        format!(
            "{} persisted surface brief(s) name a primary target that no longer exists: {}.",
            orphaned.len(),
            orphaned
                .iter()
                .map(|b| format!("{} → {}", b.path, b.primary_target.as_deref().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        "Ask whether the surface moved (repoint the brief) or was removed (delete the brief). Until then the brief is authority for a file that is gone.".to_string(),
    )]
}

/// JS: checkProjectRoots
pub fn check_project_roots(patterns: &[String], candidates_len: usize) -> Vec<Finding> {
    let positive: Vec<&String> = patterns.iter().filter(|p| !p.is_empty() && !js_trim(p).starts_with('!')).collect();
    if positive.is_empty() || candidates_len > 0 {
        return vec![];
    }
    vec![finding(
        "config-project-roots-match-nothing",
        "config.json",
        Some(".impeccable/config.json".to_string()),
        "mention",
        format!(
            "`projectRoots` declares {}, but no directory matches any of them, so the repo root is being treated as the active project.",
            positive.iter().map(|p| format!("`{}`", p)).collect::<Vec<_>>().join(", ")
        ),
        "Report the patterns and ask which directories they should name. A renamed workspace folder is the usual cause.".to_string(),
    )]
}

pub struct BootExtras {
    pub abs_design_path: Option<String>,
    pub sidecar_candidates: Vec<String>,
    pub project_root_patterns: Option<Vec<String>>,
    pub target_candidates: Vec<TargetCandidate>,
}

/// JS: collectBootFindingGroups(ctx, extras) — the boot artifact checks
/// grouped by artifact, so deeper reports (doctor) can interleave their own
/// checks without rebuilding this policy (upstream 80997663).
pub struct BootFindingGroups {
    pub product: Vec<Finding>,
    pub native_platform: Vec<Finding>,
    pub design_sidecar: Vec<Finding>,
    pub config: Vec<Finding>,
    pub build_path: Vec<Finding>,
    pub surface_briefs: Vec<Finding>,
    pub project_roots: Vec<Finding>,
}

pub fn collect_boot_finding_groups(ctx: &Ctx, cwd: &str, extras: &BootExtras) -> BootFindingGroups {
    let project_root = if ctx.project_root.is_empty() { cwd.to_string() } else { ctx.project_root.clone() };
    BootFindingGroups {
        product: check_product(ctx.product.as_deref(), ctx.product_path.as_deref().unwrap_or("PRODUCT.md")),
        // Only checked once a PRODUCT.md exists. Without one the boot already
        // emits NO_PRODUCT_MD and routes into init, which asks for the
        // platform directly; a second signal saying the same thing is noise.
        native_platform: if ctx.product.as_deref().map(|p| !p.is_empty()).unwrap_or(false) {
            check_native_platform_evidence(
                &project_root,
                ctx.platform.as_deref(),
                ctx.product.as_deref(),
                ctx.product_path.as_deref(),
            )
        } else {
            Vec::new()
        },
        design_sidecar: check_design_sidecar(extras.abs_design_path.as_deref(), &extras.sidecar_candidates, &project_root),
        config: check_config(&project_root, Some(&ctx.repo_root)),
        build_path: check_build_path_unset(&project_root, Some(&ctx.repo_root), ctx.product.as_deref()),
        surface_briefs: check_surface_briefs(&ctx.surface_brief_candidates, &project_root),
        project_roots: match &extras.project_root_patterns {
            Some(patterns) => check_project_roots(patterns, extras.target_candidates.len()),
            None => Vec::new(),
        },
    }
}

/// JS: collectBootFindings(ctx, extras)
pub fn collect_boot_findings(ctx: &Ctx, cwd: &str, extras: &BootExtras) -> Vec<Finding> {
    let groups = collect_boot_finding_groups(ctx, cwd, extras);
    let mut out = Vec::new();
    out.extend(groups.product);
    out.extend(groups.native_platform);
    out.extend(groups.design_sidecar);
    out.extend(groups.config);
    out.extend(groups.build_path);
    out.extend(groups.surface_briefs);
    out.extend(groups.project_roots);
    out
}

pub static _UNUSED: Lazy<()> = Lazy::new(|| ());

#[cfg(test)]
mod terminal_evidence_tests {
    use super::{check_native_platform_evidence, pep503_normalize};

    fn scratch(name: &str) -> String {
        let base = std::env::temp_dir().join(format!("impeccable-terminal-evidence-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base.to_string_lossy().into_owned()
    }

    fn write(root: &str, rel: &str, body: &str) {
        let p = std::path::Path::new(root).join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    const WEB_PRODUCT: &str = "# P\n\n## Platform\n\nweb\n\n## Positioning\nx\n";

    #[test]
    fn ratatui_dependency_suggests_terminal_for_a_web_project() {
        let root = scratch("ratatui");
        write(&root, "Cargo.toml", "[package]\nname = \"x\"\n\n[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].id, "platform-native-evidence");
        assert!(f[0].summary.contains("a ratatui dependency"), "{}", f[0].summary);
        assert!(f[0].summary.contains("terminal codebase"), "{}", f[0].summary);
        assert!(f[0].fix.contains("`terminal`"), "{}", f[0].fix);
    }

    #[test]
    fn go_mod_and_python_manifests_count_as_evidence() {
        let root = scratch("go");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea v1.2.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbletea dependency"), "{}", f[0].summary);

        let root = scratch("py");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"textual>=0.80\",\n]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);
    }

    #[test]
    fn go_mod_single_line_require_counts_as_evidence() {
        let root = scratch("go-single");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire github.com/charmbracelet/lipgloss v1.0.0\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a lipgloss dependency"), "{}", f[0].summary);
        assert!(f[0].fix.contains("`terminal`"), "{}", f[0].fix);
    }

    #[test]
    fn ink_in_package_json_counts_as_evidence() {
        let root = scratch("ink");
        write(&root, "package.json", "{\"dependencies\":{\"ink\":\"^5.0.0\"}}");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("an ink dependency"), "{}", f[0].summary);
        assert!(f[0].fix.contains("`terminal`"), "{}", f[0].fix);
    }

    #[test]
    fn declared_terminal_project_skips_the_check() {
        let root = scratch("declared");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("terminal"), Some("# P\n\n## Platform\n\nterminal\n"), Some("PRODUCT.md"));
        assert!(f.is_empty());
    }

    #[test]
    fn mobile_evidence_outranks_terminal_evidence() {
        let root = scratch("mixed");
        write(&root, "ios/Podfile", "platform :ios, '15.0'\n");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].fix.contains("`ios`"), "{}", f[0].fix);
        assert!(f[0].summary.contains("native codebase"), "{}", f[0].summary);
    }

    #[test]
    fn a_prefixed_crate_name_is_not_a_match() {
        let root = scratch("prefix");
        write(&root, "Cargo.toml", "[dependencies]\nratatui-macros = \"0.6\"\nratatuix = \"1\"\n");
        write(&root, "requirements.txt", "textual-dev==1.0\ntextualize==1.0\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn go_major_version_path_counts_and_an_indirect_require_does_not() {
        let root = scratch("go-v2");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea/v2 v2.0.0\n\tgithub.com/charmbracelet/lipgloss/v2 v2.0.0 // indirect\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("carries a bubbletea dependency."), "{}", f[0].summary);
        assert!(!f[0].summary.contains("lipgloss"), "{}", f[0].summary);
    }

    #[test]
    fn go_mod_blocks_other_than_require_do_not_count() {
        let root = scratch("go-blocks");
        write(
            &root,
            "go.mod",
            "module example.com/app\n\ngo 1.24\n\nreplace (\n\tgithub.com/charmbracelet/bubbletea => ../fork\n)\n\nexclude (\n\tgithub.com/charmbracelet/lipgloss v0.9.0\n)\n\nretract (\n\tv1.0.1\n)\n\ntool (\n\tcharm.land/bubbles/v2/cmd/demo\n)\n\nrequire github.com/charmbracelet/bubbletea v1.2.0 // indirect\n",
        );
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn charm_land_module_paths_and_bubbles_count_as_evidence() {
        let root = scratch("charm-land");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n\nrequire (\n\tcharm.land/bubbletea/v2 v2.0.0\n\tcharm.land/bubbles/v2 v2.0.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbletea dependency and a bubbles dependency"), "{}", f[0].summary);

        let root = scratch("bubbles-v1");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire github.com/charmbracelet/bubbles v0.20.0\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbles dependency"), "{}", f[0].summary);
    }

    #[test]
    fn a_module_on_both_charm_hosts_is_listed_once() {
        let root = scratch("both-hosts");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea v1.3.0\n\tcharm.land/bubbletea/v2 v2.0.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].summary.matches("a bubbletea dependency").count(), 1, "{}", f[0].summary);
    }

    #[test]
    fn go_replace_directive_is_not_a_dependency_line() {
        let root = scratch("go-replace");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nreplace github.com/charmbracelet/bubbletea => ../fork\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn two_mobile_platforms_plus_terminal_resolve_to_adaptive() {
        let root = scratch("mixed-three");
        write(&root, "ios/Podfile", "platform :ios, '15.0'\n");
        write(&root, "android/build.gradle", "apply plugin: 'com.android.application'\n");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].fix.contains("`adaptive`"), "{}", f[0].fix);
        assert!(f[0].fix.contains("matching native reference"), "{}", f[0].fix);
    }

    #[test]
    fn terminal_suggestion_names_the_terminal_reference() {
        let root = scratch("fix-text");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].fix.contains("matching terminal reference"), "{}", f[0].fix);
    }

    #[test]
    fn rich_alone_is_not_terminal_evidence() {
        let root = scratch("rich-req");
        write(&root, "requirements.txt", "rich==13.7.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());

        let root = scratch("rich-pyproject");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"rich>=13\",\n]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn crossterm_alone_is_not_terminal_evidence() {
        let root = scratch("crossterm");
        write(&root, "Cargo.toml", "[dependencies]\ncrossterm = \"0.28\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn a_reason_found_in_two_manifests_is_listed_once() {
        let root = scratch("textual-twice");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"textual>=0.80\",\n]\n");
        write(&root, "requirements.txt", "textual==0.80.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].summary.matches("a textual dependency").count(), 1, "{}", f[0].summary);
    }

    #[test]
    fn pep503_folds_case_and_separator_runs() {
        assert_eq!(pep503_normalize("Textual"), "textual");
        assert_eq!(pep503_normalize("Textual__Dev.-Tools"), "textual-dev-tools");
    }

    #[test]
    fn pyproject_inline_array_and_capitalized_names_count() {
        let root = scratch("py-inline");
        write(&root, "pyproject.toml", "[project]\nname = \"notes\"\ndependencies = [\"Textual>=0.80\", \"rich\"]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("carries a textual dependency."), "{}", f[0].summary);

        let root = scratch("req-extras");
        write(&root, "requirements.txt", "-r base.txt\nTextual[syntax] >= 0.80 ; python_version >= \"3.9\"\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);
    }

    #[test]
    fn poetry_key_form_still_counts() {
        let root = scratch("poetry");
        write(&root, "pyproject.toml", "[tool.poetry.dependencies]\npython = \"^3.11\"\nTextual = \"^0.80\"\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);
    }

    #[test]
    fn prose_and_comments_naming_textual_are_not_dependencies() {
        let root = scratch("py-prose");
        write(
            &root,
            "pyproject.toml",
            "[project]\nname = \"notes\"\ndescription = \"Textual app for managing notes\"\ndependencies = [\"httpx>=0.27\"]  # textual>=0.80 later\n",
        );
        write(&root, "requirements.txt", "# textual==0.80.0\nhttpx==0.27.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn cargo_dotted_keys_and_dependency_headers_count() {
        let bodies = [
            "[dependencies]\nratatui.workspace = true\n",
            "[dependencies]\nratatui.version = \"0.29\"\n",
            "[dependencies.ratatui]\nversion = \"0.29\"\n",
            "[target.'cfg(unix)'.dependencies.ratatui]\nversion = \"0.29\"\n",
            "[workspace.dependencies]\n\"ratatui\" = \"0.29\" # tui\n",
        ];
        for (i, body) in bodies.iter().enumerate() {
            let root = scratch(&format!("cargo-form-{i}"));
            write(&root, "Cargo.toml", body);
            let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
            assert_eq!(f.len(), 1, "{body}");
            assert!(f[0].summary.contains("a ratatui dependency"), "{body}: {}", f[0].summary);
        }
    }

    #[test]
    fn ratatui_named_only_in_package_metadata_is_not_a_dependency() {
        let root = scratch("cargo-metadata");
        write(
            &root,
            "Cargo.toml",
            "[package]\nname = \"ratatui-notes\"\ndescription = \"ratatui demo\"\nkeywords = [\"ratatui\"]\n\n[dependencies]\ncrossterm = \"0.28\"\n",
        );
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }
}
