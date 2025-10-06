use cocoatly_core::error::{CocoatlyError, Result};
use std::fs::{self, File, Metadata};
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use walkdir::WalkDir;

pub struct FileSystemOps;

impl FileSystemOps {
    pub fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::create_dir_all(path.as_ref())?;
        Ok(())
    }

    pub fn remove_directory<P: AsRef<Path>>(path: P) -> Result<()> {
        if path.as_ref().exists() {
            fs::remove_dir_all(path.as_ref())?;
        }
        Ok(())
    }

    pub fn copy_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        source: P,
        destination: Q,
    ) -> Result<Vec<PathBuf>> {
        let source_path = source.as_ref();
        let dest_path = destination.as_ref();

        if !source_path.exists() {
            return Err(CocoatlyError::IoError(
                io::Error::new(io::ErrorKind::NotFound, "Source directory not found")
            ));
        }

        Self::ensure_directory(dest_path)?;

        let mut copied_files = Vec::new();

        for entry in WalkDir::new(source_path) {
            let entry = entry?;
            let path = entry.path();

            let relative_path = path.strip_prefix(source_path)
                .map_err(|e| CocoatlyError::IoError(
                    io::Error::new(io::ErrorKind::Other, e.to_string())
                ))?;

            let target_path = dest_path.join(relative_path);

            if path.is_dir() {
                Self::ensure_directory(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    Self::ensure_directory(parent)?;
                }
                fs::copy(path, &target_path)?;
                copied_files.push(target_path);
            }
        }

        Ok(copied_files)
    }

    pub fn move_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        source: P,
        destination: Q,
    ) -> Result<()> {
        fs::rename(source.as_ref(), destination.as_ref())?;
        Ok(())
    }

    pub fn list_files<P: AsRef<Path>>(directory: P) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(directory.as_ref()) {
            let entry = entry?;
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        Ok(files)
    }

    pub fn get_directory_size<P: AsRef<Path>>(directory: P) -> Result<u64> {
        let mut total_size = 0u64;

        for entry in WalkDir::new(directory.as_ref()) {
            let entry = entry?;
            if entry.path().is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }

    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
        let mut file = File::open(path.as_ref())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            Self::ensure_directory(parent)?;
        }

        let mut file = File::create(path.as_ref())?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }

    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }

    pub fn directory_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_dir()
    }

    pub fn get_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
        let metadata = fs::metadata(path.as_ref())?;
        Ok(metadata)
    }

    pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(
        original: P,
        link: Q,
    ) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original.as_ref(), link.as_ref())?;
        }

        #[cfg(windows)]
        {
            if original.as_ref().is_dir() {
                std::os::windows::fs::symlink_dir(original.as_ref(), link.as_ref())?;
            } else {
                std::os::windows::fs::symlink_file(original.as_ref(), link.as_ref())?;
            }
        }

        Ok(())
    }
}

pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    FileSystemOps::ensure_directory(path)
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    FileSystemOps::remove_directory(path)
}

pub fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q) -> Result<Vec<PathBuf>> {
    FileSystemOps::copy_directory(source, dest)
}
