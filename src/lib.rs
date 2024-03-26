use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, path::PathBuf};
mod extra;

#[cfg(feature = "backend")]
pub mod backend;
#[cfg(feature = "frontend")]
pub mod frontend;

pub mod collections;
pub mod msg;
pub mod search;

pub(crate) mod ser_de;

/// Custom file type which is just a wrapper around the std `PathBuf` for cross-platform serialization and deserialization.
#[derive(PartialEq)]
pub struct PitouFilePath {
    pub path: PathBuf,
}

impl PitouFilePath {
    pub fn name(&self) -> &str {
        if self.path.as_os_str().len() == 0 { return "Drives" }
        let res = self.path
            .file_name()
            .map(|v| v.to_str().unwrap_or_default())
            .unwrap_or_default();
        res
    }

    pub fn from_pathbuf(pathbuf: PathBuf) -> Self {
        Self { path: pathbuf }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.path.as_os_str().as_encoded_bytes()
    }

    pub fn ancestors(&self) -> impl Iterator<Item = PitouFilePath> {
        let mut ll = std::collections::LinkedList::new();
        for anc in self.path.ancestors() {
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

    pub fn format_as_dir_entries(&self) -> String {
        format!("{} items", self.bytes)
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
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, PitouFileKind::Directory)
    }

    fn attempt(path: &PathBuf) -> Option<Self> {
        std::fs::metadata(path).map(|v| v.into()).ok()
    }
}

pub struct PitouDrive {
    pub name: String,
    pub mount_point: PitouFilePath,
    pub total_space: u64,
    pub free_space: u64,
    pub is_removable: bool,
    pub kind: PitouDriveKind,
}

impl PartialEq for PitouDrive {
    fn eq(&self, other: &Self) -> bool {
        &self.mount_point == &other.mount_point
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PitouDriveKind {
    HDD,
    SSD,
    Unknown,
}

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
        self.path.name()
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum PitouFileSortOrder {
    Increasing,
    Decreasing,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum PitouFileSort {
    DateCreated(PitouFileSortOrder),
    Name(PitouFileSortOrder),
    DateModified(PitouFileSortOrder),
    DateAccessed(PitouFileSortOrder),
}

impl PitouFileSort {
    pub fn sorted(self, mut items: Vec<PitouFile>) -> Vec<PitouFile> {
        match self {
            PitouFileSort::DateCreated(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.created.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.created.datetime))
                }),
            },
            PitouFileSort::Name(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by(|a, b| a.name().cmp(&b.name()))
                }
                PitouFileSortOrder::Decreasing => {
                    items.sort_unstable_by(|a, b| b.name().cmp(&a.name()))
                }
            },
            PitouFileSort::DateModified(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.modified.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.modified.datetime))
                }),
            },
            PitouFileSort::DateAccessed(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.accessed.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.accessed.datetime))
                }),
            },
        }
        items
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PitouFileFilter {
    pub files: bool,
    pub links: bool,
    pub dirs: bool,
}

impl PitouFileFilter {
    pub fn new() -> Self {
        Self {
            files: true,
            links: false,
            dirs: true,
        }
    }

    pub fn include_all() -> Self {
        Self {
            files: true,
            links: true,
            dirs: true,
        }
    }

    pub fn map(self, file: PitouFile) -> Option<PitouFile> {
        if (file.is_dir() && self.include_dirs())
            || (file.is_file() && self.include_files())
            || (file.is_link() && self.include_links())
        {
            Some(file)
        } else {
            None
        }
    }

    pub fn all_filtered(self) -> bool {
        !self.dirs && !self.files && !self.links
    }

    pub fn include_dirs(self) -> bool {
        self.dirs
    }

    pub fn include_files(self) -> bool {
        self.files
    }

    pub fn include_links(self) -> bool {
        self.links
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write! {f, "rgba({}, {}, {}, {})", self.0, self.1, self.2, self.3}
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ColorTheme {
    pub background1: Color,
    pub background2: Color,
    pub foreground1: Color,
    pub foreground2: Color,
    pub spare1: Color,
    pub spare2: Color,
}

impl ColorTheme {
    pub const RAMBO: Self = Self {
        background1: Color(230, 230, 230, 255),
        background2: Color(180, 180, 180, 255),
        foreground1: Color(50, 50, 50, 255),
        foreground2: Color(80, 80, 80, 255),
        spare1: Color(215, 215, 215, 255),
        spare2: Color(150, 150, 150, 255),
    };

    pub const DEFAULT_DARK: Self = Self {
        background1: Color(25, 25, 112, 255),
        background2: Color(0, 0, 51, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(192, 192, 192, 255),
        spare1: Color(153, 50, 204, 255),
        spare2: Color(255, 0, 0, 255),
    };

    pub const POLISH: Self = Self {
        background1: Color(30, 30, 30, 255),
        background2: Color(60, 60, 60, 255),
        foreground1: Color(220, 220, 220, 255),
        foreground2: Color(180, 180, 180, 255),
        spare1: Color(45, 45, 45, 255),
        spare2: Color(100, 100, 100, 255),
    };

    pub const GPT_DARK: Self = Self {
        background1: Color(50, 50, 50, 255),
        background2: Color(105, 105, 105, 255),
        foreground1: Color(240, 240, 240, 255),
        foreground2: Color(255, 255, 255, 255),
        spare1: Color(0, 0, 0, 255),
        spare2: Color(185, 210, 235, 255),
    };

    pub const OCEAN_BLUE: Self = Self {
        background1: Color(100, 200, 255, 255),
        background2: Color(50, 100, 200, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(245, 225, 180, 255),
        spare1: Color(25, 50, 100, 255),
        spare2: Color(150, 15, 50, 255),
    };

    pub const GEM_LIGHT: Self = Self {
        background1: Color(240, 240, 240, 255),
        background2: Color(200, 200, 200, 255),
        foreground1: Color(50, 50, 50, 255),
        foreground2: Color(0, 128, 128, 255),
        spare1: Color(170, 170, 170, 255),
        spare2: Color(255, 165, 0, 255),
    };

    pub const GEM_DARK: Self = Self {
        background1: Color(50, 50, 50, 255),
        background2: Color(30, 30, 30, 255),
        foreground1: Color(240, 240, 240, 255),
        foreground2: Color(0, 255, 255, 255),
        spare1: Color(100, 100, 100, 255),
        spare2: Color(255, 192, 203, 255),
    };
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppMenu {
    Home,
    Explorer,
    Trash,
    Favorites,
    Search,
    Locked,
    Recents,
    Cloud,
    Settings,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GeneralFolder {
    DocumentsFolder(PitouFilePath),
    AudiosFolder(PitouFilePath),
    PicturesFolder(PitouFilePath),
    VideosFolder(PitouFilePath),
    DesktopFolder(PitouFilePath),
    DownloadsFolder(PitouFilePath),
}

impl GeneralFolder {
    pub fn o_name(&self) -> &str {
        match self {
            GeneralFolder::DocumentsFolder(d) => d.name(),
            GeneralFolder::AudiosFolder(a) => a.name(),
            GeneralFolder::PicturesFolder(p) => p.name(),
            GeneralFolder::VideosFolder(v) => v.name(),
            GeneralFolder::DesktopFolder(d) => d.name(),
            GeneralFolder::DownloadsFolder(d) => d.name(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            GeneralFolder::DocumentsFolder(_) => String::from("Documents"),
            GeneralFolder::AudiosFolder(_) => String::from("Audios"),
            GeneralFolder::PicturesFolder(_) => String::from("pictures"),
            GeneralFolder::VideosFolder(_) => String::from("Videos"),
            GeneralFolder::DesktopFolder(_) => String::from("Desktop"),
            GeneralFolder::DownloadsFolder(_) => String::from("Downloads"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ItemsView {
    Grid,
    Rows,
    Tiles,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub refresh_rate: u8,
    pub show_extensions: bool,
    pub single_click_opens: bool,
    pub hide_impermisible: bool,
    pub show_thumbnails: bool,
    pub items_view: ItemsView,
    pub show_parents: bool,
    pub items_zoom: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            refresh_rate: 250,
            show_extensions: true,
            single_click_opens: false,
            hide_impermisible: true,
            show_thumbnails: false,
            items_view: ItemsView::Rows,
            show_parents: false,
            items_zoom: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DirChild {
    name: String,
    metadata: Option<PitouFileMetadata>,
}