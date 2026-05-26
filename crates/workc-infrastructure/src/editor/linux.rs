use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{EditorError, EditorLauncher};
use workc_domain::editor::EditorRegistry;

pub struct LinuxEditorLauncher {
    registry: EditorRegistry,
}

impl LinuxEditorLauncher {
    pub fn new() -> Self {
        Self {
            registry: EditorRegistry::new(),
        }
    }
}

impl EditorLauncher for LinuxEditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: &str) -> Result<(), EditorError> {
        let cmd = self.registry.find(editor).map(|e| e.launch_cmd()).unwrap_or(editor);

        Command::new(cmd)
            .arg(path.as_str())
            .spawn()
            .map_err(|error| EditorError {
                detail: format!("failed to launch {cmd}: {error}"),
            })?;

        Ok(())
    }
}
