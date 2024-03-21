use crate::{
    search::{SearchOptions, SearchType},
    PitouFile, PitouFileFilter,
};
use std::{path::PathBuf, sync::Arc};

impl SearchType {
    pub(crate) fn matches(&self, input: &str, sensitive: bool) -> bool {
        match self {
            Self::Regex(pattern) => pattern.is_match(input),
            Self::MatchBegining(key) => {
                if sensitive {
                    input.starts_with(key)
                } else {
                    Self::starts_with_ignore_case(key, input)
                }
            }
            Self::MatchMiddle(key) => {
                if sensitive {
                    input.contains(key)
                } else {
                    Self::contains_ignore_case(key, input)
                }
            }
            Self::MatchEnding(key) => {
                if sensitive {
                    input.ends_with(key)
                } else {
                    Self::ends_with_ignore_case(key, input)
                }
            }
        }
    }

    fn starts_with_ignore_case(key: &str, input: &str) -> bool {
        if input.len() < key.len() {
            return false;
        }
        (0..key.len()).all(|i| {
            let (v, u) = (key.as_bytes()[i], input.as_bytes()[i]);
            let fc = if v > 96 && v < 123 { v - 32 } else { v };
            let sc = if u > 96 && u < 123 { u - 32 } else { u };
            fc == sc
        })
    }

    fn ends_with_ignore_case(key: &str, input: &str) -> bool {
        if input.len() < key.len() {
            return false;
        }
        (0..key.len()).all(|i| {
            let (v, u) = (
                key.as_bytes()[key.len() - i - 1],
                input.as_bytes()[input.len() - i - 1],
            );
            let fc = if v > 96 && v < 123 { v - 32 } else { v };
            let sc = if u > 96 && u < 123 { u - 32 } else { u };
            fc == sc
        })
    }

    fn contains_ignore_case(key: &str, input: &str) -> bool {
        if input.len() < key.len() {
            return false;
        }
        (0..=(input.len() - key.len())).any(|b| {
            (0..key.len()).all(|i| {
                let (v, u) = (key.as_bytes()[i], input.as_bytes()[b + i]);
                let fc = if v > 96 && v < 123 { v - 32 } else { v };
                let sc = if u > 96 && u < 123 { u - 32 } else { u };
                fc == sc
            })
        })
    }
}

pub mod stream {
    use std::{collections::LinkedList, sync::OnceLock};

    use crate::{frontend::msg::SearchMsg, PitouFile};
    use tokio::{sync::Mutex, task::JoinHandle};

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
        get_stream()
            .lock()
            .await
            .as_mut()
            .map(|l| SearchMsg::Active(l.split_off(0)))
            .unwrap_or(SearchMsg::Terminated(LinkedList::new()))
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
            search_dir.path.path.clone(),
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
