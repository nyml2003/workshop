use std::fs;
use std::io;

use camino::Utf8Path;
use camino::Utf8PathBuf;

use super::file_system::FileSystem;

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn clone_box(&self) -> Box<dyn FileSystem> {
        Box::new(RealFileSystem)
    }

    fn read_to_string(&self, path: &Utf8Path) -> io::Result<String> {
        fs::read_to_string(path.as_std_path())
    }

    fn write(&self, path: &Utf8Path, contents: &str) -> io::Result<()> {
        fs::write(path.as_std_path(), contents)
    }

    fn create_dir_all(&self, path: &Utf8Path) -> io::Result<()> {
        fs::create_dir_all(path.as_std_path())
    }

    fn read_dir(&self, path: &Utf8Path) -> io::Result<Vec<Utf8PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path.as_std_path())? {
            let entry = entry?;
            let name = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                io::Error::new(io::ErrorKind::InvalidData, path.display().to_string())
            })?;
            entries.push(name);
        }
        Ok(entries)
    }

    fn remove_dir_all(&self, path: &Utf8Path) -> io::Result<()> {
        fs::remove_dir_all(path.as_std_path())
    }

    fn copy_file(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
        fs::copy(from.as_std_path(), to.as_std_path())?;
        Ok(())
    }

    fn copy_dir(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
        self.create_dir_all(to)?;
        for entry in self.read_dir(from)? {
            let name = entry.file_name().unwrap_or_default();
            let target = to.join(name);
            if self.is_dir(&entry) {
                self.copy_dir(&entry, &target)?;
            } else {
                self.copy_file(&entry, &target)?;
            }
        }
        Ok(())
    }

    fn exists(&self, path: &Utf8Path) -> bool {
        path.as_std_path().exists()
    }

    fn is_dir(&self, path: &Utf8Path) -> bool {
        path.as_std_path().is_dir()
    }
}
