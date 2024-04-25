use std::{
    fs::{FileType, Metadata},
    hash::Hash,
    path::PathBuf,
    time::SystemTime,
};

use chrono::DateTime;

use crate::{
    PitouDateTime, PitouFile, PitouFileKind, PitouFileMetadata, PitouFilePath, PitouFileSize,
};

impl From<u64> for PitouFileSize {
    fn from(bytes: u64) -> Self {
        Self { bytes }
    }
}

impl From<PathBuf> for PitouFilePath {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl From<FileType> for PitouFileKind {
    fn from(value: FileType) -> Self {
        if value.is_dir() {
            Self::Directory
        } else if value.is_file() {
            Self::File
        } else {
            Self::Link
        }
    }
}

impl From<SystemTime> for PitouDateTime {
    fn from(value: SystemTime) -> Self {
        let millis_epoch = value
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Self {
            datetime: DateTime::from_timestamp_millis(millis_epoch)
                .unwrap()
                .naive_utc(),
        }
    }
}

impl From<Metadata> for PitouFileMetadata {
    fn from(value: Metadata) -> Self {
        Self {
            modified: value.modified().unwrap().into(),
            accessed: value.accessed().unwrap().into(),
            created: value.created().unwrap().into(),
            size: value.len().into(),
            kind: value.file_type().into(),
        }
    }
}

impl PitouFile {
    pub fn new(path: PathBuf, metadata: Metadata) -> Self {
        let path = path.into();
        let metadata = metadata.try_into().ok();
        Self { path, metadata }
    }
}

impl Hash for PitouFilePath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.path.as_os_str().as_encoded_bytes())
    }
}

impl PartialEq for PitouFile {
    fn eq(&self, other: &Self) -> bool {
        &self.path == &other.path
    }
}

impl AsRef<std::path::Path> for PitouFile {
    fn as_ref(&self) -> &std::path::Path {
        &self.path.path
    }
}
