use std::path::PathBuf;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
mod ser_de;
mod search;
mod extra;

/// Custom file type which is just a wrapper around the std `PathBuf` for cross-platform serialization and deserialization.
pub struct PitouFilePath {
    pub path: PathBuf,
}


#[derive(Serialize, Deserialize)]
pub struct PitouDateTime {
    pub datetime: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub enum PitouFileKind {
    Directory,
    File,
    Link
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

pub struct PitouFile {
    pub path: PitouFilePath,
    pub metadata: PitouFileMetadata,
}

impl PitouFile {
    pub fn is_dir(&self) -> bool {
        matches!(self.metadata.kind, PitouFileKind::Directory)
    }
    pub fn is_link(&self) -> bool {
        matches!(self.metadata.kind, PitouFileKind::Link)
    }
    pub fn is_file(&self) -> bool {
        matches!(self.metadata.kind, PitouFileKind::File)
    }

    pub fn name(&self) -> &str {
        self.path.path.file_name().unwrap().to_str().unwrap()
    }
}


#[test]
fn test_serialization_n_deserialization() {
    let path = PitouFilePath { path: PathBuf::from("D:/workspace/pitou") };
    println!("original path: {}", path.path.display());
    let serialized_str = serde_json::to_string(&path).unwrap();
    println!("json form : {}", serialized_str);
    let path = serde_json::from_str::<PitouFilePath>(&serialized_str).unwrap();
    println!("deserialized path: {}", path.path.display());
}