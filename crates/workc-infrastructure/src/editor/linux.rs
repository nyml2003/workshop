use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{EditorError, EditorKind, EditorLauncher};

pub struct LinuxEditorLauncher;

impl EditorLauncher for LinuxEditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: EditorKind) -> Result<(), EditorError> {
        let name = match &editor {
            EditorKind::Cursor => "cursor",
            EditorKind::VsCode => "code",
            EditorKind::Other(value) => value.as_str(),
        };

        Command::new(name)
            .arg(path.as_str())
            .spawn()
            .map_err(|error| EditorError {
                detail: format!("failed to launch {name}: {error}"),
            })?;

        Ok(())
    }
}
