use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
mod extra;

#[cfg(feature = "backend")]
pub mod backend;

pub mod search;

pub mod frontend;
mod ser_de;

/// Custom file type which is just a wrapper around the std `PathBuf` for cross-platform serialization and deserialization.
#[derive(PartialEq)]
pub struct PitouFilePath {
    pub path: PathBuf,
}

impl PitouFilePath {
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default()
    }

    pub fn from_pathbuf(pathbuf: PathBuf) -> Self {
        Self { path: pathbuf }
    }

    pub fn ancestors(&self) -> impl Iterator<Item = PitouFilePath> {
        let mut ll = std::collections::LinkedList::new();
        for anc in self.path.ancestors() {
            if anc.as_os_str().len() == 0 {
                break;
            }
            ll.push_front(PitouFilePath::from_pathbuf(std::path::PathBuf::from(anc)))
        }
        ll.into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PitouDateTime {
    pub datetime: NaiveDateTime,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PitouFileKind {
    Directory,
    File,
    Link,
}

#[derive(Clone, Copy)]
pub struct PitouFileSize {
    bytes: u64,
}

impl PitouFileSize {
    const KB: f64 = (2u64 << 10) as f64;
    const MB: f64 = (2u64 << 20) as f64;
    const GB: f64 = (2u64 << 30) as f64;
    const TB: f64 = (2u64 << 40) as f64;
    pub fn format(self) -> String {
        let bytes = self.bytes as f64;
        if bytes < Self::KB {
            format! {"{:.2} B", bytes}
        } else if bytes < Self::MB {
            format! {"{:.2} KB", bytes / Self::KB }
        } else if bytes < Self::GB {
            format! {"{:.2} MB", bytes / Self::MB }
        } else if bytes < Self::TB {
            format! {"{:.2} GB", bytes / Self::GB }
        } else {
            format! {"{:.2} TB", bytes / Self::TB }
        }
    }

    pub fn new(value: u64) -> Self {
        Self { bytes: value }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PitouFileMetadata {
    pub modified: PitouDateTime,
    pub accessed: PitouDateTime,
    pub created: PitouDateTime,
    pub size: PitouFileSize,
    pub kind: PitouFileKind,
}

impl PitouFileMetadata {
    fn attempt(path: &PathBuf) -> Option<Self> {
        std::fs::metadata(path).map(|v| v.into()).ok()
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PitouDrive {
    pub name: String,
    pub mount_point: PitouFilePath,
    pub total_space: u64,
    pub free_space: u64,
    pub is_removable: bool,
    pub kind: PitouDriveKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PitouDriveKind {
    HDD,
    SSD,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct PitouFile {
    pub path: PitouFilePath,
    pub metadata: Option<PitouFileMetadata>,
}

impl PitouFile {
    pub fn from_pathbuf(path: PathBuf) -> Self {
        Self {
            metadata: PitouFileMetadata::attempt(&path),
            path: path.into(),
        }
    }

    pub fn is_dir(&self) -> bool {
        match &self.metadata {
            None => false,
            Some(metadata) => matches!(metadata.kind, PitouFileKind::Directory),
        }
    }
    pub fn is_link(&self) -> bool {
        match &self.metadata {
            None => false,
            Some(metadata) => matches!(metadata.kind, PitouFileKind::Link),
        }
    }
    pub fn is_file(&self) -> bool {
        match &self.metadata {
            None => false,
            Some(metadata) => matches!(metadata.kind, PitouFileKind::File),
        }
    }

    pub fn name(&self) -> &str {
        self.path.path.file_name().unwrap().to_str().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PitouTrashItem {
    pub original_path: PitouFilePath,
    pub metadata: PitouTrashItemMetadata,
}

impl PitouTrashItem {
    pub fn name(&self) -> &str {
        self.original_path.name()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PitouTrashItemMetadata {
    pub id: String,
    pub deleted: PitouDateTime,
}
