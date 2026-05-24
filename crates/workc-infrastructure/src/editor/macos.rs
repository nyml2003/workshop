use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{EditorError, EditorKind, EditorLauncher};

pub struct MacOsEditorLauncher;

impl EditorLauncher for MacOsEditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: EditorKind) -> Result<(), EditorError> {
        let command = match editor {
            EditorKind::Cursor => "cursor",
            EditorKind::VsCode => "code",
            EditorKind::Other(ref value) => value.as_str(),
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
