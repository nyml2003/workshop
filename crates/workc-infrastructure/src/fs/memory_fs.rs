use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;

use camino::Utf8Component;
use camino::Utf8Path;
use camino::Utf8PathBuf;

use super::file_system::{FileSystem, FsCall};

#[derive(Debug)]
enum FsNode {
    File(String),
    Directory(BTreeMap<String, FsNode>),
}

impl Default for FsNode {
    fn default() -> Self {
        FsNode::Directory(BTreeMap::new())
    }
}

#[derive(Debug, Default)]
struct MemoryFsState {
    root: FsNode,
    ops: Vec<FsCall>,
}

impl MemoryFsState {
    fn new() -> Self {
        Self {
            root: FsNode::Directory(BTreeMap::new()),
            ops: Vec::new(),
        }
    }
}

pub struct MemoryFileSystem {
    state: Rc<RefCell<MemoryFsState>>,
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(MemoryFsState::new())),
        }
    }

    pub fn ops(&self) -> Vec<FsCall> {
        self.state.borrow().ops.clone()
    }

    fn components(path: &Utf8Path) -> Vec<String> {
        path.components()
            .filter_map(|c| match c {
                Utf8Component::RootDir => None,
                Utf8Component::CurDir => None,
                Utf8Component::ParentDir => None,
                Utf8Component::Normal(name) => Some(name.to_string()),
                _ => None,
            })
            .collect()
    }

    fn get_node<'a>(root: &'a mut FsNode, parts: &[String]) -> Option<&'a mut FsNode> {
        let mut current = root;
        for part in parts {
            match current {
                FsNode::Directory(children) => {
                    current = children.get_mut(part.as_str())?;
                }
                FsNode::File(_) => return None,
            }
        }
        Some(current)
    }

    fn ensure_dirs<'a>(root: &'a mut FsNode, parts: &[String]) -> &'a mut FsNode {
        let mut current = root;
        for part in parts {
            match current {
                FsNode::Directory(children) => {
                    if !children.contains_key(part.as_str()) {
                        children.insert(part.clone(), FsNode::Directory(BTreeMap::new()));
                    }
                    current = children.get_mut(part.as_str()).unwrap();
                }
                FsNode::File(_) => {
                    let new_dir = FsNode::Directory(BTreeMap::new());
                    *current = new_dir;
                }
            }
        }
        current
    }
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemoryFileSystem {
    fn clone_box(&self) -> Box<dyn FileSystem> {
        Box::new(MemoryFileSystem {
            state: self.state.clone(),
        })
    }

    fn read_to_string(&self, path: &Utf8Path) -> io::Result<String> {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        state.ops.push(FsCall::ReadToStr(path.to_owned()));

        match Self::get_node(&mut state.root, &parts) {
            Some(FsNode::File(content)) => Ok(content.clone()),
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("file not found: {path}"),
            )),
        }
    }

    fn write(&self, path: &Utf8Path, contents: &str) -> io::Result<()> {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        let bytes = contents.len();
        state.ops.push(FsCall::Write(path.to_owned(), bytes));

        if parts.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot write to root",
            ));
        }

        let (parent_parts, name) = parts.split_at(parts.len() - 1);
        let parent = Self::ensure_dirs(&mut state.root, parent_parts);

        match parent {
            FsNode::Directory(children) => {
                children.insert(name[0].clone(), FsNode::File(contents.to_owned()));
                Ok(())
            }
            FsNode::File(_) => Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("parent is a file: {path}"),
            )),
        }
    }

    fn create_dir_all(&self, path: &Utf8Path) -> io::Result<()> {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        state.ops.push(FsCall::CreateDirAll(path.to_owned()));

        Self::ensure_dirs(&mut state.root, &parts);
        Ok(())
    }

    fn read_dir(&self, path: &Utf8Path) -> io::Result<Vec<Utf8PathBuf>> {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        state.ops.push(FsCall::ReadDir(path.to_owned()));

        match Self::get_node(&mut state.root, &parts) {
            Some(FsNode::Directory(children)) => {
                let entries: Vec<Utf8PathBuf> = children
                    .keys()
                    .map(|name| {
                        if parts.is_empty() {
                            Utf8PathBuf::from(name)
                        } else {
                            path.join(name)
                        }
                    })
                    .collect();
                Ok(entries)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("directory not found: {path}"),
            )),
        }
    }

    fn remove_dir_all(&self, path: &Utf8Path) -> io::Result<()> {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        state.ops.push(FsCall::RemoveDirAll(path.to_owned()));

        if parts.is_empty() {
            state.root = FsNode::Directory(BTreeMap::new());
            return Ok(());
        }

        let (parent_parts, name) = parts.split_at(parts.len() - 1);
        match Self::get_node(&mut state.root, parent_parts) {
            Some(FsNode::Directory(children)) => {
                children.remove(&name[0]);
                Ok(())
            }
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("not found: {path}"),
            )),
        }
    }

    fn copy_file(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        state
            .ops
            .push(FsCall::CopyFile(from.to_owned(), to.to_owned()));

        let content = {
            let parts = Self::components(from);
            match Self::get_node(&mut state.root, &parts) {
                Some(FsNode::File(content)) => content.clone(),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("source not found: {from}"),
                    ));
                }
            }
        };

        drop(state);

        self.write(to, &content)
    }

    fn copy_dir(&self, from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        state
            .ops
            .push(FsCall::CopyDir(from.to_owned(), to.to_owned()));

        let from_parts = Self::components(from);
        match Self::get_node(&mut state.root, &from_parts) {
            Some(FsNode::Directory(_)) => (),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("source directory not found: {from}"),
                ));
            }
        };

        drop(state);

        self.create_dir_all(to)?;
        let entries = self.read_dir(from)?;
        for entry in &entries {
            let name = entry.file_name().unwrap_or_default();
            let target = to.join(name);
            if self.is_dir(entry) {
                self.copy_dir(entry, &target)?;
            } else {
                self.copy_file(entry, &target)?;
            }
        }

        Ok(())
    }

    fn exists(&self, path: &Utf8Path) -> bool {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        let exists = Self::get_node(&mut state.root, &parts).is_some();
        state.ops.push(FsCall::Exists(path.to_owned(), exists));
        exists
    }

    fn is_dir(&self, path: &Utf8Path) -> bool {
        let parts = Self::components(path);
        let mut state = self.state.borrow_mut();
        let is_dir = matches!(
            Self::get_node(&mut state.root, &parts),
            Some(FsNode::Directory(_))
        );
        state.ops.push(FsCall::IsDir(path.to_owned(), is_dir));
        is_dir
    }
}
