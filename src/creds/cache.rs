use crate::UsernamePasswordPair;
use std::{path::PathBuf, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StatError {
    #[error("Failed to stat cache file at {path:?}: {source}")]
    StatFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to get modification time for cache file at {path:?}: {source}")]
    ModifiedFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to calculate age of cache file at {path:?}: {source}")]
    ElapsedFailed {
        path: PathBuf,
        source: std::time::SystemTimeError,
    },
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error(transparent)]
    StatFailed(#[from] StatError),
    #[error("Failed to read cache file at {path:?}: {source}")]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to deserialize credentials from cache file at {path:?}: {source}")]
    DeserializeFailed {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Failed to create cache directory at {path:?}: {source}")]
    CreateDirFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to serialize credentials: {0}")]
    SerializeFailed(#[from] serde_json::Error),
    #[error("Failed to write cache file at {path:?}: {source}")]
    WriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
#[error("Failed to delete cache file at {path:?}: {source}")]
pub struct DeleteError {
    path: PathBuf,
    source: std::io::Error,
}

// 60 minutes is the actual expiration on GitHub, but we need to have a margin
// to ensure the token doesn't expire mid-build.
const MAX_TOKEN_AGE: Duration = Duration::from_secs(45 * 60);

#[derive(Debug)]
pub struct Cache {
    file: PathBuf,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            file: std::env::temp_dir()
                .join(concat!("com.brainium.", env!("CARGO_PKG_NAME")))
                .join("creds.json"),
        }
    }

    fn stale(&self) -> Result<bool, StatError> {
        log::info!("checking creds file at {:?}", self.file);
        if self.file.is_file() {
            let age = std::fs::metadata(&self.file)
                .map_err(|source| StatError::StatFailed {
                    path: self.file.clone(),
                    source,
                })?
                .modified()
                .map_err(|source| StatError::ModifiedFailed {
                    path: self.file.clone(),
                    source,
                })?
                .elapsed()
                .map_err(|source| StatError::ElapsedFailed {
                    path: self.file.clone(),
                    source,
                })?;
            log::info!("current creds file is {} minutes old", age.as_secs() / 60);
            Ok(age >= MAX_TOKEN_AGE)
        } else {
            log::info!("no creds file currently exists");
            Ok(true)
        }
    }

    pub fn read(&self) -> Result<Option<UsernamePasswordPair>, ReadError> {
        if self.stale()? {
            Ok(None)
        } else {
            log::info!("reading creds file at {:?}", self.file);
            let bytes = std::fs::read(&self.file).map_err(|source| ReadError::ReadFailed {
                path: self.file.clone(),
                source,
            })?;
            serde_json::from_slice(&bytes).map(Some).map_err(|source| {
                ReadError::DeserializeFailed {
                    path: self.file.clone(),
                    source,
                }
            })
        }
    }

    pub fn write(&self, creds: &UsernamePasswordPair) -> Result<(), WriteError> {
        log::info!("updating creds file at {:?}", self.file);
        let dir = self
            .file
            .parent()
            .expect("developer error: cache file path doesn't have a parent");
        if !dir.is_dir() {
            std::fs::create_dir_all(&dir).map_err(|source| WriteError::CreateDirFailed {
                path: dir.to_owned(),
                source,
            })?;
        }
        let bytes = serde_json::to_vec(creds)?;
        std::fs::write(&self.file, bytes).map_err(|source| WriteError::WriteFailed {
            path: self.file.clone(),
            source,
        })?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), DeleteError> {
        log::info!("deleting creds file at {:?}", self.file);
        if self.file.is_file() {
            std::fs::remove_file(&self.file).map_err(|source| DeleteError {
                path: self.file.clone(),
                source,
            })?;
        }
        Ok(())
    }
}
