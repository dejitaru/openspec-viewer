use std::fs;
use std::path::Path;
use std::sync::Mutex;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenSpecEntry {
    path: String,
    relative: String,
    section: String,
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
        .invoke_handler(tauri::generate_handler![
            read_openspec_dir,
            read_file_content,
            save_file_content,
            set_root
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn sample_root() -> PathBuf {
        let root = std::env::temp_dir().join("openspec-visualizer-test");
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
        let root = sample_root();
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
        let root = sample_root();
        let mut watcher =
            notify::recommended_watcher(|_: Result<notify::Event, notify::Error>| {})
                .expect("fallo al crear watcher");
        watcher
            .watch(Path::new(&root), RecursiveMode::Recursive)
            .expect("fallo al iniciar watch");
        let _ = fs::remove_dir_all(&root);
    }
}
