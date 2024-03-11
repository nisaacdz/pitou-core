use std::path::PathBuf;

use crate::{PitouFile, PitouFilePath};
mod drive;
pub use drive::*;
use serde::{Deserialize, Serialize};

pub mod clipboard {
    use std::sync::{Arc, OnceLock};

    use tokio::sync::Mutex;

    enum ClipboardItem {
        Copied(Arc<Vec<PitouFile>>),
        Cut(Arc<Vec<PitouFile>>)
    }

    use crate::PitouFile;
    type QUEUE = Mutex<Vec<ClipboardItem>>;

    static CLIPBOARD: OnceLock<QUEUE> = OnceLock::new();

    fn get_clipboard() -> &'static QUEUE {
        CLIPBOARD.get_or_init(|| Mutex::new(Vec::new()))
    }

    pub async fn copy(files: Vec<PitouFile>) {
        get_clipboard().lock().await.push(ClipboardItem::Copied(Arc::new(files)))
    }

    pub async fn cut(files: Vec<PitouFile>) {
        get_clipboard().lock().await.push(ClipboardItem::Cut(Arc::new(files)))
    }
    
    pub async fn remove_from_clipboard(idx: usize) {
        get_clipboard().lock().await.remove(idx);
    }

    pub async fn clear_clipboard() {
        get_clipboard().lock().await.clear()
    }

    pub async fn is_empty() -> bool {
        get_clipboard().lock().await.is_empty()
    }

    pub async fn paste() -> Option<Arc<Vec<PitouFile>>> {
        let cb = get_clipboard();
        let mut guard = cb.lock().await;
        let items = match guard.pop() {
            Some(v) => match v {
                ClipboardItem::Copied(vals) => vals,
                ClipboardItem::Cut(vals) => vals,
            },
            None => return None
        };
        guard.push(ClipboardItem::Copied(items.clone()));
        std::mem::drop(guard);
        Some(items)
    }
}

pub async fn delete(items: Vec<PitouFilePath>) {
    for item in items {
        tokio::spawn(async move { trash::delete(&item.path) });
    }
}

pub fn open(file: PitouFilePath) -> std::io::Result<()> {
    open::that_detached(&file.path)
}

pub fn share(_file: PitouFilePath) -> std::io::Result<()> {
    todo!()
}

pub async fn rename(file: PitouFilePath, newname: String) {
    let newpath = file.path.parent().unwrap_or(&PathBuf::new()).join(newname);
    tokio::fs::rename(&file.path, newpath).await.unwrap();
}

pub async fn create_file(file: PitouFilePath) {
    tokio::fs::File::create(&file.path).await.expect("couldn't create file");
}

pub async fn create_dir(dir: PitouFilePath) {
    tokio::fs::create_dir(&dir.path).await.expect("couldn't create dir");
}

pub async fn read_link(link: PitouFilePath) -> Option<crate::PitouFile> {
    tokio::fs::read_link(&link.path)
        .await
        .map(|path| PitouFile::from_pathbuf(path)).ok()
}

fn downloads_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::download_dir().unwrap())
}

fn desktop_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::desktop_dir().unwrap())
}

fn videos_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::video_dir().unwrap())
}

fn pictures_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::picture_dir().unwrap())
}

fn audios_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::audio_dir().unwrap())
}

fn documents_folder() -> PitouFilePath {
    PitouFilePath::from_pathbuf(dirs::document_dir().unwrap())
}

#[derive(Serialize, Deserialize)]
pub enum GeneralFolders {
    DocumentsFolder(PitouFilePath),
    AudiosFolder(PitouFilePath),
    PicturesFolder(PitouFilePath),
    VideosFolder(PitouFilePath),
    DesktopFolder(PitouFilePath),
    DownloadsFolder(PitouFilePath),
}

pub fn general_folders() -> Vec<GeneralFolders> {
    vec![
        GeneralFolders::DesktopFolder(desktop_folder()),
        GeneralFolders::DownloadsFolder(downloads_folder()),
        GeneralFolders::AudiosFolder(audios_folder()),
        GeneralFolders::VideosFolder(videos_folder()),
        GeneralFolders::PicturesFolder(pictures_folder()),
        GeneralFolders::DocumentsFolder(documents_folder())
    ]
}