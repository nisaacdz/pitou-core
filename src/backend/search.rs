use std::{path::PathBuf, sync::Arc};
use crate::{PitouFile, search::{SearchOptions, SearchType}, frontend::PitouFileFilter};

pub mod stream {
    use std::{collections::LinkedList, sync::OnceLock};

    use tokio::{sync::Mutex, task::JoinHandle};
    use crate::{PitouFile, frontend::msg::SearchMsg};

    type QUEUE = Mutex<Option<LinkedList<PitouFile>>>;
    type SPAWNS = Mutex<LinkedList<JoinHandle<()>>>;

    static HANDLES: OnceLock<SPAWNS> = OnceLock::new();
    static STREAM: OnceLock<QUEUE> = OnceLock::new();

    fn get_handles() -> &'static SPAWNS {
        HANDLES.get_or_init(|| Mutex::new(LinkedList::new()))
    }

    fn get_stream() -> &'static QUEUE {
        STREAM.get_or_init(|| Mutex::new(None))
    }

    pub async fn is_active() -> bool {
        get_stream().lock().await.is_some()
    }

    pub async fn terminate_stream() {
        get_stream().lock().await.take();
    }

    pub async fn begin_stream() {
        let _ = get_stream().lock().await.insert(LinkedList::new());
    }

    pub async fn read() -> SearchMsg {
        get_stream().lock().await.as_mut().map(|l| SearchMsg::Active(l.split_off(0))).unwrap_or(SearchMsg::Terminated(LinkedList::new()))
    }

    pub async fn write(find: PitouFile) {
        get_stream()
            .lock()
            .await
            .as_mut()
            .map(|l| l.push_back(find));
    }

    pub async fn append_handle(handle: JoinHandle<()>) {
        get_handles().lock().await.push_back(handle);
    }

    pub async fn abort() {
        for handle in get_handles().lock().await.split_off(0).into_iter().rev() {
            handle.abort()
        }
    }

    pub async fn wait_for_all_ops() {
        for handle in get_handles().lock().await.split_off(0).into_iter().rev() {
            let _ = handle.await;
        }
    }
}

#[allow(unused)]
#[derive(Clone)]
struct SearchVariables {
    filter: PitouFileFilter,
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
    let hardware_accelerate = options.hardware_accelerate;
    let (variables, directory) = options.into();
    if variables.filter.all_filtered() {
        return;
    }
    stream::begin_stream().await;
    tokio::spawn(async move {
        recursive_search(directory, variables).await;
        stream::terminate_stream().await;
        if hardware_accelerate {
            stream::wait_for_all_ops().await;
        }
    });
}

#[async_recursion::async_recursion]
async fn recursive_search(directory: PathBuf, mut variables: SearchVariables) {
    if variables.depth == 0 || !stream::is_active().await {
        return;
    }
    variables.depth -= 1;
    while let Ok(Some(de)) = tokio::fs::read_dir(&directory)
        .await
        .unwrap()
        .next_entry()
        .await
    {
        let file = PitouFile::new(de.path(), de.metadata().await.unwrap());
        if file.is_dir() {
            let vclone = variables.clone();
            stream::append_handle(tokio::spawn(async move {
                recursive_search(de.path(), vclone).await
            }))
            .await;
        }
        if variables.include(&file) {
            stream::write(file).await;
        }
    }
}
