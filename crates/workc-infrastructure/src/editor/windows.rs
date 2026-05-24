use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{EditorError, EditorKind, EditorLauncher};

pub struct WindowsEditorLauncher;

impl EditorLauncher for WindowsEditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: EditorKind) -> Result<(), EditorError> {
        let command = match &editor {
            EditorKind::Cursor => resolve_windows_command("cursor")?,
            EditorKind::VsCode => resolve_windows_command("code")?,
            EditorKind::Other(value) => resolve_windows_command(value)?,
        };

        Command::new(command)
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
            detail: "program not found".to_owned(),
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
        detail: "program not found".to_owned(),
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
