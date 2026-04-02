pub mod view;

use std::path::PathBuf;

// ── App entry ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    /// Shell-ready exec string (field codes stripped).
    pub exec: String,
    /// XDG icon name (reserved for Phase 7 icon rendering).
    pub icon: String,
}

// ── XDG .desktop scanner ──────────────────────────────────────────────────────

/// Scans XDG application directories and returns a sorted list of launchable apps.
///
/// Directories searched (in XDG precedence order):
///   $HOME/.local/share/applications
///   /usr/share/applications
pub fn load_apps() -> Vec<AppEntry> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    dirs.push(PathBuf::from("/usr/share/applications"));

    let mut apps: Vec<AppEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            if let Some(app) = parse_desktop_file(&path) {
                // Local entries shadow system ones with the same name.
                if seen.insert(app.name.to_lowercase()) {
                    apps.push(app);
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

/// Parses a single .desktop file, returning `None` if not launchable.
fn parse_desktop_file(path: &std::path::Path) -> Option<AppEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut in_desktop_entry = false;
    let mut app_type = String::new();
    let mut name = String::new();
    let mut exec = String::new();
    let mut icon = String::new();
    let mut no_display = false;
    let mut hidden = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }

        if let Some(v) = line.strip_prefix("Type=") {
            app_type = v.to_owned();
        } else if let Some(v) = line.strip_prefix("Name=") {
            if name.is_empty() {
                name = v.to_owned();
            }
        } else if let Some(v) = line.strip_prefix("Exec=") {
            exec = strip_field_codes(v);
        } else if let Some(v) = line.strip_prefix("Icon=") {
            icon = v.to_owned();
        } else if line == "NoDisplay=true" {
            no_display = true;
        } else if line == "Hidden=true" {
            hidden = true;
        }
    }

    if app_type != "Application" || no_display || hidden || name.is_empty() || exec.is_empty() {
        return None;
    }

    Some(AppEntry { name, exec, icon })
}

/// Strips XDG Exec field codes (%f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %%→%).
fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            match chars.next() {
                Some('%') => out.push('%'),
                Some(_)   => {} // drop all other field codes
                None      => {}
            }
        } else {
            out.push(ch);
        }
    }
    out.trim().to_owned()
}

// ── Fuzzy filter ─────────────────────────────────────────────────────────────

/// Returns up to 8 entries matching `query`, ranked by match quality.
pub fn fuzzy_filter<'a>(apps: &'a [AppEntry], query: &str) -> Vec<&'a AppEntry> {
    if query.is_empty() {
        return apps.iter().take(8).collect();
    }

    let q = query.to_lowercase();

    let mut scored: Vec<(usize, &AppEntry)> = apps
        .iter()
        .filter_map(|app| {
            let name_lc = app.name.to_lowercase();
            if name_lc == q {
                Some((0, app))  // exact
            } else if name_lc.starts_with(&q) {
                Some((1, app))  // prefix
            } else if name_lc.contains(q.as_str()) {
                // rank by how early the match appears
                let pos = name_lc.find(q.as_str()).unwrap_or(usize::MAX);
                Some((2 + pos, app))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by_key(|(score, _)| *score);
    scored.into_iter().take(8).map(|(_, app)| app).collect()
}
