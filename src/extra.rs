use std::{fs::{FileType, Metadata}, path::PathBuf, time::SystemTime};

use chrono::NaiveDateTime;

use crate::{PitouDateTime, PitouFile, PitouFileKind, PitouFileMetadata, PitouFilePath, PitouFileSize};

impl From<u64> for PitouFileSize {
    fn from(bytes : u64) -> Self {
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
        let millis_epoch = value.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()as i64;
        Self {
            datetime: NaiveDateTime::from_timestamp_millis(millis_epoch).unwrap(),
        }
    }
}

impl TryFrom<Metadata> for PitouFileMetadata {
    type Error = std::io::Error;
    fn try_from(value: Metadata) -> Result<PitouFileMetadata, Self::Error> {
        let res = Self {
            modified: value.modified()?.into(),
            accessed: value.accessed()?.into(),
            created: value.created()?.into(),
            size: value.len().into(),
            kind: value.file_type().into()
        };
        Ok(res)
    }
}

impl PitouFile {
    pub fn new(path: PathBuf, metadata: Metadata) -> Self {
        let path = path.into();
        let metadata = metadata.try_into().ok();
        Self { path, metadata }
    }
}