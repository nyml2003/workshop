use std::io;

use camino::Utf8Path;
use camino::Utf8PathBuf;

pub trait FileSystem {
    fn clone_box(&self) -> Box<dyn FileSystem>;
    fn read_to_string(&self, path: &Utf8Path) -> io::Result<String>;
    fn write(&self, path: &Utf8Path, contents: &str) -> io::Result<()>;
    fn create_dir_all(&self, path: &Utf8Path) -> io::Result<()>;
    fn read_dir(&self, path: &Utf8Path) -> io::Result<Vec<Utf8PathBuf>>;
    fn remove_dir_all(&self, path: &Utf8Path) -> io::Result<()>;
    fn copy_file(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()>;
    fn copy_dir(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()>;
    fn exists(&self, path: &Utf8Path) -> bool;
    fn is_dir(&self, path: &Utf8Path) -> bool;
}

#[derive(Debug, Clone)]
pub enum FsCall {
    ReadToStr(Utf8PathBuf),
    Write(Utf8PathBuf, usize),
    CreateDirAll(Utf8PathBuf),
    ReadDir(Utf8PathBuf),
    RemoveDirAll(Utf8PathBuf),
    CopyFile(Utf8PathBuf, Utf8PathBuf),
    CopyDir(Utf8PathBuf, Utf8PathBuf),
    Exists(Utf8PathBuf, bool),
    IsDir(Utf8PathBuf, bool),
}
