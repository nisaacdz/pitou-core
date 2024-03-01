use std::{path::PathBuf, sync::Arc};

use tokio::task::JoinHandle;

use crate::PitouFile;

use super::{FileFilter, SearchOptions, SearchType};

pub mod stream {
    use std::{
        collections::LinkedList,
        sync::{Arc, OnceLock},
    };

    pub enum StreamWriteResult {
        Success,
        StreamTerminated,
    }

    use tokio::sync::Mutex;

    use crate::PitouFile;
    type QUEUE = Arc<Mutex<Option<LinkedList<PitouFile>>>>;

    static STREAM: OnceLock<QUEUE> = OnceLock::new();

    fn get_stream() -> QUEUE {
        STREAM.get_or_init(|| Arc::new(Mutex::new(None))).clone()
    }

    pub async fn terminate_stream() {
        get_stream().lock().await.take();
    }

    pub async fn begin_stream() {
        let _ = get_stream().lock().await.insert(LinkedList::new());
    }

    pub async fn read() -> Option<LinkedList<PitouFile>> {
        get_stream().lock().await.as_mut().map(|l| l.split_off(0))
    }

    pub async fn write(find: PitouFile) -> StreamWriteResult {
        get_stream()
            .lock()
            .await
            .as_mut()
            .map(|l| {
                l.push_back(find);
                StreamWriteResult::Success
            })
            .unwrap_or(StreamWriteResult::StreamTerminated)
    }
}

#[derive(Clone)]
struct SearchVariables {
    filter: FileFilter,
    case_sensitive: bool,
    depth: u8,
    search_type: Arc<SearchType>,
    skip_errors: bool,
}

impl From<SearchOptions> for (SearchVariables, PathBuf) {
    fn from(value: SearchOptions) -> Self {
        let SearchOptions {
            search_dir,
            hardware_accelerate: _,
            filter,
            case_sensitive,
            depth,
            search_type,
            skip_errors,
            max_finds: _,
        } = value;
        (
            SearchVariables {
                filter,
                case_sensitive,
                depth,
                skip_errors,
                search_type: Arc::new(search_type),
            },
            search_dir.path,
        )
    }
}

impl SearchVariables {
    fn include(&self, file: &PitouFile) -> bool {
        ((file.is_file() && self.filter.include_files())
            || (file.is_dir() && self.filter.include_dirs())
            || (file.is_link() && self.filter.include_links()))
            && self.search_type.matches(file.name(), self.case_sensitive)
    }
}

#[allow(unused)]
pub async fn search(options: SearchOptions) {
    let (variables, directory) = options.into();
    if variables.filter.all_filtered() {
        return;
    }
    stream::begin_stream().await;
    tokio::spawn(async {
        recursive_search(directory, variables).await;
        stream::terminate_stream().await
    });
}

#[async_recursion::async_recursion]
async fn recursive_search(directory: PathBuf, mut variables: SearchVariables) {
    if variables.depth == 0 {
        return;
    }
    variables.depth -= 1;
    let mut spawns = Vec::new();
    while let Ok(Some(de)) = tokio::fs::read_dir(&directory)
        .await
        .unwrap()
        .next_entry()
        .await
    {
        let file = PitouFile::new(de.path(), de.metadata().await.unwrap());
        if variables.include(&file) {
            if let stream::StreamWriteResult::StreamTerminated = stream::write(file).await {
                return safe_abort(spawns);
            }
        }
        let vclone = variables.clone();
        spawns.push(tokio::spawn(async move {
            recursive_search(de.path(), vclone).await
        }))
    }
    safe_return(spawns).await
}

#[inline]
fn safe_abort(spawns: Vec<JoinHandle<()>>) {
    for handle in spawns {
        handle.abort();
    }
}

#[inline]
async fn safe_return(spawns: Vec<JoinHandle<()>>) {
    for handle in spawns {
        match handle.await {
            Ok(_) => (),
            Err(e) => {
                if !e.is_cancelled() {
                    panic!("some unknown error caused panic!")
                }
            }
        }
    }
}
