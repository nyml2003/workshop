use anyhow::Result;
use camino::Utf8PathBuf;
use workc_infrastructure::fs::{FileSystem, RealFileSystem};

use super::shared::workspace_root;

pub struct CliContext {
    pub fs: Box<dyn FileSystem>,
    pub workspace_root: Utf8PathBuf,
}

impl CliContext {
    pub fn production() -> Result<Self> {
        Ok(Self {
            fs: Box::new(RealFileSystem),
            workspace_root: workspace_root()?,
        })
    }

    #[allow(dead_code)]
    pub fn new(fs: Box<dyn FileSystem>, root: Utf8PathBuf) -> Self {
        Self {
            fs,
            workspace_root: root,
        }
    }
}
