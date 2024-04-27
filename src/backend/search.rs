use std::{path::PathBuf, rc::Rc, sync::Arc};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{search::SimplifiedSearchOptions, PitouFile, PitouFileFilter};

impl SimplifiedSearchOptions {
    pub fn try_into(self) -> Option<SearchOptions> {
        if let Some(search_type) = SearchType::parse_regex(self.search_kind, self.input) {
            let obj = SearchOptions {
                search_dir: self.search_dir,
                filter: self.filter,
                case_sensitive: self.case_sensitive,
                hardware_accelerate: self.hardware_accelerate,
                skip_errors: self.skip_errors,
                depth: self.depth,
                max_finds: self.max_finds,
                search_type: search_type,
            };
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SearchType {
    #[serde(with = "serde_regex")]
    Regex(Regex),
    MatchBegining(String),
    MatchMiddle(String),
    MatchEnding(String),
}

impl SearchType {
    pub(crate) fn parse_regex(search_kind: u8, search_key: String) -> Option<Self> {
        match search_kind {
            0 => Some(SearchType::MatchBegining(search_key)),
            1 => Some(SearchType::MatchEnding(search_key)),
            2 => Some(SearchType::MatchMiddle(search_key)),
            _ => regex::Regex::new(&search_key)
                .map(|r| SearchType::Regex(r))
                .ok(),
        }
    }
}

pub struct SearchOptions {
    pub(crate) search_dir: Rc<PitouFile>,
    pub(crate) hardware_accelerate: bool,
    pub(crate) filter: PitouFileFilter,
    pub(crate) case_sensitive: bool,
    pub(crate) depth: u8,
    pub(crate) search_type: SearchType,
    pub(crate) skip_errors: bool,
    pub(crate) max_finds: usize,
}

impl SearchOptions {
    pub fn new(search_dir: Rc<PitouFile>, key: String) -> Self {
        Self {
            search_dir,
            filter: PitouFileFilter::new(),
            hardware_accelerate: false,
            case_sensitive: true,
            depth: 6,
            search_type: SearchType::MatchMiddle(key),
            skip_errors: true,
            max_finds: 100,
        }
    }
}

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

    use crate::{msg::SearchMsg, PitouFile};
    use tokio::{sync::Mutex, task::JoinHandle};

    type COUNT = Mutex<Option<usize>>;
    type QUEUE = Mutex<Option<LinkedList<PitouFile>>>;
    type SPAWNS = Mutex<LinkedList<JoinHandle<()>>>;

    static HANDLES: OnceLock<SPAWNS> = OnceLock::new();
    static STREAM: OnceLock<QUEUE> = OnceLock::new();
    static FINDS: OnceLock<COUNT> = OnceLock::new();

    fn get_finds() -> &'static COUNT {
        FINDS.get_or_init(|| Mutex::new(None))
    }

    fn get_handles() -> &'static SPAWNS {
        HANDLES.get_or_init(|| Mutex::new(LinkedList::new()))
    }

    fn get_stream() -> &'static QUEUE {
        STREAM.get_or_init(|| Mutex::new(None))
    }

    /// decrements the count and returns true if the max_finds has not yet been exhusted
    /// Automatically closes the finds if the count has dropped to zero.
    async fn count_and_proceed() -> bool {
        match &mut *get_finds().lock().await {
            Some(count) => {
                if *count == 0 {
                    false
                } else {
                    *count -= 1;
                    true
                }
            }
            None => false,
        }
    }

    /// checks if the strema was ended abruptly from outside
    pub async fn is_terminated() -> bool {
        get_stream().lock().await.is_none()
    }

    #[allow(unused)]
    /// checks if the stream has completed its task
    async fn has_finished() -> bool {
        get_finds().lock().await.is_none()
    }

    /// used for ending the stream from within
    async fn finish_stream() {
        get_finds().lock().await.take();
    }

    /// used for ending the stream from outside
    pub async fn terminate_stream() {
        get_stream().lock().await.take();
    }

    pub async fn begin_stream(max_finds: usize) {
        tokio::join! {
            async move { let _ = get_stream().lock().await.insert(LinkedList::new()); },
            async move { let _ = get_finds().lock().await.insert(max_finds); }
        };
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
        if count_and_proceed().await {
            get_stream()
                .lock()
                .await
                .as_mut()
                .map(|l| l.push_back(find));
        } else {
            tokio::join! {
                finish_stream(),
                abort_remaining_ops()
            };
        }
    }

    pub async fn append_handle(handle: JoinHandle<()>) {
        get_handles().lock().await.push_back(handle);
    }

    pub async fn abort_remaining_ops() {
        for handle in get_handles().lock().await.split_off(0).into_iter().rev() {
            handle.abort()
        }
    }

    //TODO erroneous code leads to forever wait
    pub async fn wait_for_all_ops() {
        // for handle in get_handles().lock().await.split_off(0).into_iter().rev() {
        //     let _ = handle.await;
        // }
        ()
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

pub async fn search(options: SearchOptions) {
    let hardware_accelerate = options.hardware_accelerate;
    let max_finds = options.max_finds;
    let (variables, directory) = options.into();
    if variables.filter.all_filtered() {
        return;
    }
    stream::begin_stream(max_finds).await;
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
    if variables.depth == 0 || stream::is_terminated().await {
        return;
    }
    variables.depth -= 1;
    let mut read_dir = if let Ok(read_dir) = tokio::fs::read_dir(&directory).await {
        read_dir
    } else {
        return;
    };

    while let Ok(Some(de)) = read_dir.next_entry().await {
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
