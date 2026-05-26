//! Infrastructure layer for workc.

pub mod editor;
pub mod fs;
pub mod git;
pub mod time;

pub use fs::file_system::{FileSystem, FsCall};
pub use fs::memory_fs::MemoryFileSystem;
pub use fs::real_fs::RealFileSystem;
