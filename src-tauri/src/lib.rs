use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenSpecEntry {
    path: String,
    relative: String,
    section: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    path: String,
    name: String,
    icon: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    theme: String,
    projects: Vec<Project>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "one-dark-pro".to_string(),
            projects: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeFile {
    id: String,
    label: String,
    colors: BTreeMap<String, String>,
}

#[allow(dead_code)]
struct WatchHandle(Mutex<RecommendedWatcher>);

fn scan_section(section: &str, root: &Path) -> Vec<OpenSpecEntry> {
    let dir = root.join("openspec").join(section);
    let mut entries = Vec::new();
    if !dir.exists() {
        return entries;
    }
    for entry in WalkDir::new(&dir).follow_links(true).min_depth(1) {
        let Ok(entry) = entry else { continue };
        let is_md = entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str()) == Some("md");
        if !is_md {
            continue;
        }
        let path = entry.path().to_string_lossy().to_string();
        let relative = entry
            .path()
            .strip_prefix(&dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.clone());
        entries.push(OpenSpecEntry {
            path,
            relative,
            section: section.to_string(),
        });
    }
    entries.sort_by(|a, b| a.relative.to_lowercase().cmp(&b.relative.to_lowercase()));
    entries
}

#[tauri::command]
fn read_openspec_dir(root: String) -> Vec<OpenSpecEntry> {
    let root = Path::new(&root);
    let mut entries = scan_section("specs", root);
    entries.extend(scan_section("changes", root));
    entries
}

#[tauri::command]
fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_file_content(path: String, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

fn storage_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?
        .join("storage");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
fn storage_path(app: AppHandle) -> Result<String, String> {
    storage_dir(&app).map(|d| d.to_string_lossy().to_string())
}

fn theme_colors(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn builtin_themes() -> Vec<ThemeFile> {
    vec![
        ThemeFile {
            id: "one-dark-pro".to_string(),
            label: "One Dark Pro".to_string(),
            colors: theme_colors(&[
                ("slate-950", "#1e2227"),
                ("slate-900", "#21252b"),
                ("slate-800", "#2c313a"),
                ("slate-700", "#3e4451"),
                ("slate-600", "#4b5263"),
                ("slate-500", "#5c6370"),
                ("slate-400", "#7f848e"),
                ("slate-300", "#abb2bf"),
                ("slate-200", "#c0c5ce"),
                ("slate-100", "#d7dae0"),
                ("emerald-300", "#98c379"),
                ("emerald-400", "#98c379"),
                ("emerald-500", "#7fb069"),
                ("emerald-600", "#5c8a46"),
                ("emerald-700", "#456d33"),
                ("amber-100", "#e5c07b"),
                ("amber-200", "#e5c07b"),
                ("amber-300", "#e5c07b"),
                ("amber-400", "#e5c07b"),
                ("amber-500", "#c7a05e"),
                ("amber-800", "#8a6d2f"),
                ("amber-900", "#6b531f"),
                ("amber-950", "#4a3a12"),
                ("red-300", "#e06c75"),
                ("red-800", "#b04a52"),
                ("red-950", "#7a2d33"),
                ("cyan-300", "#56b6c2"),
                ("cyan-400", "#3d9daa"),
            ]),
        },
        ThemeFile {
            id: "dracula".to_string(),
            label: "Dracula".to_string(),
            colors: theme_colors(&[
                ("slate-950", "#191a21"),
                ("slate-900", "#21222c"),
                ("slate-800", "#282a36"),
                ("slate-700", "#44475a"),
                ("slate-600", "#505466"),
                ("slate-500", "#6272a4"),
                ("slate-400", "#8a8fa8"),
                ("slate-300", "#bfc2d1"),
                ("slate-200", "#e0e0ea"),
                ("slate-100", "#f8f8f2"),
                ("emerald-300", "#50fa7b"),
                ("emerald-400", "#50fa7b"),
                ("emerald-500", "#34c95e"),
                ("emerald-600", "#27a04a"),
                ("emerald-700", "#1c7937"),
                ("amber-100", "#f1fa8c"),
                ("amber-200", "#f1fa8c"),
                ("amber-300", "#f1fa8c"),
                ("amber-400", "#f1fa8c"),
                ("amber-500", "#c4cd63"),
                ("amber-800", "#828845"),
                ("amber-900", "#62662f"),
                ("amber-950", "#41441e"),
                ("red-300", "#ff5555"),
                ("red-800", "#c43d3d"),
                ("red-950", "#872828"),
                ("cyan-300", "#8be9fd"),
                ("cyan-400", "#6fd6eb"),
            ]),
        },
        ThemeFile {
            id: "tokyo-night".to_string(),
            label: "Tokyo Night".to_string(),
            colors: theme_colors(&[
                ("slate-950", "#16161e"),
                ("slate-900", "#1a1b26"),
                ("slate-800", "#24283b"),
                ("slate-700", "#292e42"),
                ("slate-600", "#3b4261"),
                ("slate-500", "#565f89"),
                ("slate-400", "#737aa2"),
                ("slate-300", "#9aa5ce"),
                ("slate-200", "#a9b1d6"),
                ("slate-100", "#c0caf5"),
                ("emerald-300", "#9ece6a"),
                ("emerald-400", "#9ece6a"),
                ("emerald-500", "#7fae52"),
                ("emerald-600", "#62873d"),
                ("emerald-700", "#46642c"),
                ("amber-100", "#e0af68"),
                ("amber-200", "#e0af68"),
                ("amber-300", "#e0af68"),
                ("amber-400", "#e0af68"),
                ("amber-500", "#c08c4d"),
                ("amber-800", "#82602f"),
                ("amber-900", "#61491f"),
                ("amber-950", "#403013"),
                ("red-300", "#f7768e"),
                ("red-800", "#c3556d"),
                ("red-950", "#863a4b"),
                ("cyan-300", "#7dcfff"),
                ("cyan-400", "#5fb8d1"),
            ]),
        },
    ]
}

fn themes_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = storage_dir(app)?.join("themes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn seed_default_themes(dir: &Path) {
    let has_json = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        })
        .unwrap_or(false);
    if has_json {
        return;
    }
    for theme in builtin_themes() {
        if let Ok(raw) = serde_json::to_string_pretty(&theme) {
            let _ = fs::write(dir.join(format!("{}.json", theme.id)), raw);
        }
    }
}

fn themes_from_dir(dir: &Path) -> Vec<ThemeFile> {
    let mut themes = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(theme) = serde_json::from_str::<ThemeFile>(&raw) else {
                continue;
            };
            if theme.id.trim().is_empty() || theme.colors.is_empty() {
                continue;
            }
            themes.push(theme);
        }
    }
    if themes.is_empty() {
        themes = builtin_themes();
    }
    themes.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    themes
}

#[tauri::command]
fn list_themes(app: AppHandle) -> Vec<ThemeFile> {
    let Ok(dir) = themes_dir(&app) else {
        return builtin_themes();
    };
    seed_default_themes(&dir);
    themes_from_dir(&dir)
}

fn config_file(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(storage_dir(app)?.join("projects.json"))
}

#[tauri::command]
fn load_config(app: AppHandle) -> AppConfig {
    let Ok(file) = config_file(&app) else {
        return AppConfig::default();
    };
    let raw = match fs::read_to_string(&file) {
        Ok(raw) => raw,
        Err(_) => {
            let legacy = app
                .path()
                .app_config_dir()
                .ok()
                .map(|d| d.join("projects.json"));
            let Some(legacy) = legacy else {
                return AppConfig::default();
            };
            match fs::read_to_string(&legacy) {
                Ok(raw) => raw,
                Err(_) => return AppConfig::default(),
            }
        }
    };
    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&raw) {
        return cfg;
    }
    if let Ok(projects) = serde_json::from_str::<Vec<Project>>(&raw) {
        return AppConfig {
            theme: "one-dark-pro".to_string(),
            projects,
        };
    }
    AppConfig::default()
}

#[tauri::command]
fn save_config(app: AppHandle, theme: String, projects: Vec<Project>) -> Result<(), String> {
    let file = config_file(&app)?;
    let cfg = AppConfig { theme, projects };
    let raw = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    fs::write(file, raw).map_err(|e| e.to_string())
}

fn start_watcher(app: &AppHandle, root: String) -> Result<(), String> {
    let handle = app.clone();
    let mut watcher = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if !event.paths.is_empty() {
                    let paths: Vec<String> = event
                        .paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let _ = handle.emit("fs-change", paths);
                }
            }
        },
    )
    .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&root), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    app.manage(WatchHandle(Mutex::new(watcher)));
    Ok(())
}

#[tauri::command]
fn set_root(app: AppHandle, root: String) -> Result<(), String> {
    start_watcher(&app, root)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_openspec_dir,
            read_file_content,
            save_file_content,
            set_root,
            load_config,
            save_config,
            storage_path,
            list_themes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn sample_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("openspec-visualizer-test-{name}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("openspec/specs")).unwrap();
        fs::create_dir_all(root.join("openspec/changes/feature-x")).unwrap();
        fs::write(
            root.join("openspec/specs/accounts.md"),
            "# Accounts\n- [ ] tarea pendiente\n- [x] tarea hecha\n\n## Open Questions\n¿Qué pasa?\n",
        )
        .unwrap();
        fs::write(
            root.join("openspec/changes/feature-x/tasks.md"),
            "# Tasks\n- [ ] hacer algo\n",
        )
        .unwrap();
        fs::write(root.join("openspec/changes/feature-x/spec.md"), "# Spec\n").unwrap();
        root
    }

    #[test]
    fn scan_finds_specs_and_changes() {
        let root = sample_root("scan");
        let entries = read_openspec_dir(root.to_string_lossy().to_string());
        let has_spec = entries
            .iter()
            .any(|e| e.section == "specs" && e.relative == "accounts.md");
        let has_change = entries
            .iter()
            .any(|e| e.section == "changes" && e.relative == "feature-x/spec.md");
        assert!(has_spec, "spec no encontrada: {entries:?}");
        assert!(has_change, "change no encontrada: {entries:?}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn watcher_starts_on_macos() {
        let root = sample_root("watcher");
        let mut watcher =
            notify::recommended_watcher(|_: Result<notify::Event, notify::Error>| {})
                .expect("fallo al crear watcher");
        watcher
            .watch(Path::new(&root), RecursiveMode::Recursive)
            .expect("fallo al iniciar watch");
        let _ = fs::remove_dir_all(&root);
    }

    fn themes_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("openspec-visualizer-themes-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn seed_default_themes_writes_builtins_once() {
        let dir = themes_test_dir("seed");
        seed_default_themes(&dir);
        let themes = themes_from_dir(&dir);
        assert_eq!(themes.len(), builtin_themes().len());
        assert!(themes.iter().any(|t| t.id == "dracula"));

        // A user editing/removing a builtin should not be resurrected on next load.
        fs::remove_file(dir.join("dracula.json")).unwrap();
        seed_default_themes(&dir);
        let themes = themes_from_dir(&dir);
        assert!(!themes.iter().any(|t| t.id == "dracula"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn themes_from_dir_picks_up_custom_theme_and_ignores_invalid_files() {
        let dir = themes_test_dir("custom");
        fs::write(
            dir.join("solarized.json"),
            r##"{"id":"solarized","label":"Solarized","colors":{"slate-950":"#002b36"}}"##,
        )
        .unwrap();
        fs::write(dir.join("not-a-theme.json"), "{ this is not json }").unwrap();
        fs::write(dir.join("notes.txt"), "ignore me").unwrap();

        let themes = themes_from_dir(&dir);
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].id, "solarized");
        assert_eq!(themes[0].colors.get("slate-950").unwrap(), "#002b36");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn themes_from_dir_falls_back_to_builtins_when_empty() {
        let dir = themes_test_dir("empty");
        let themes = themes_from_dir(&dir);
        assert_eq!(themes.len(), builtin_themes().len());
        let _ = fs::remove_dir_all(&dir);
    }
}
