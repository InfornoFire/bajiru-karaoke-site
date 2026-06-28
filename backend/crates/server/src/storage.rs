use std::path::{Component, Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid path")]
    InvalidPath,
}

pub struct SavedFile {
    pub public_url: String,
    pub internal_path: String,
}

#[derive(Clone)]
pub struct FileStore {
    base_path: PathBuf,
    base_url: String,
}

impl FileStore {
    pub fn new(base_path: impl Into<PathBuf>, base_url: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
            base_url: base_url.into(),
        }
    }

    pub async fn save(
        &self,
        subfolder: &str,
        extension: &str,
        data: &[u8],
    ) -> Result<SavedFile, StorageError> {
        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let relative = format!("{subfolder}/{filename}");
        let full_path = self.base_path.join(&relative);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full_path, data).await?;
        Ok(SavedFile {
            public_url: format!("{}/{}", self.base_url.trim_end_matches('/'), relative),
            internal_path: full_path.to_string_lossy().into_owned(),
        })
    }

    pub async fn delete(&self, internal_path: &str) -> Result<(), StorageError> {
        let path = PathBuf::from(internal_path);
        if !self.is_within_base(&path) {
            return Err(StorageError::InvalidPath);
        }
        tokio::fs::remove_file(&path).await?;
        Ok(())
    }

    /// Resolves `..` components without requiring the path to exist, then
    /// checks the result is rooted inside `base_path`.
    fn is_within_base(&self, path: &Path) -> bool {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    normalized.pop();
                }
                Component::Normal(c) => normalized.push(c),
                Component::RootDir => normalized.push("/"),
                Component::Prefix(p) => normalized.push(p.as_os_str()),
                Component::CurDir => {}
            }
        }
        normalized.starts_with(&self.base_path)
    }
}
