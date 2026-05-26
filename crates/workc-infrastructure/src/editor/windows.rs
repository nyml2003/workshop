use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{EditorError, EditorLauncher};
use workc_domain::editor::EditorRegistry;

pub struct WindowsEditorLauncher {
    registry: EditorRegistry,
}

impl WindowsEditorLauncher {
    pub fn new() -> Self {
        Self {
            registry: EditorRegistry::new(),
        }
    }
}

impl EditorLauncher for WindowsEditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: &str) -> Result<(), EditorError> {
        let cmd = self.registry.find(editor).map(|e| e.launch_cmd()).unwrap_or(editor);
        let exe = resolve_windows_command(cmd)?;

        Command::new(exe)
            .arg(path.as_str())
            .spawn()
            .map_err(|error| EditorError {
                detail: error.to_string(),
            })?;

        Ok(())
    }
}

fn resolve_windows_command(name: &str) -> Result<String, EditorError> {
    if let Some(path) = known_windows_editor_path(name) {
        return Ok(path);
    }

    let output = Command::new("where.exe")
        .arg(name)
        .output()
        .map_err(|error| EditorError {
            detail: format!("failed to resolve command {name}: {error}"),
        })?;

    if !output.status.success() {
        return Err(EditorError {
            detail: format!("program not found: {name}"),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut candidates: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if let Some(cmd_path) = candidates
        .iter()
        .find(|candidate| candidate.to_ascii_lowercase().ends_with(".cmd"))
        .cloned()
    {
        return Ok(cmd_path);
    }

    candidates.pop().ok_or(EditorError {
        detail: format!("program not found: {name}"),
    })
}

fn known_windows_editor_path(name: &str) -> Option<String> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let candidate = match name {
        "cursor" => format!(r"{local_app_data}\Programs\cursor\resources\app\bin\cursor.cmd"),
        "code" => format!(r"{local_app_data}\Programs\Microsoft VS Code\bin\code.cmd"),
        _ => return None,
    };

    std::path::Path::new(&candidate)
        .exists()
        .then_some(candidate)
}
