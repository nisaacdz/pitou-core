use serde::{Serialize, Deserialize};
use regex::Regex;
pub mod fxns;

#[derive(Clone, Serialize, Deserialize)]
enum SearchType {
    #[serde(with = "serde_regex")]
    Regex(Regex),
    MatchBegining(String),
    MatchMiddle(String),
    MatchEnding(String),
}

impl SearchType {
    fn matches(&self, input: &str) -> bool {
        match self {
            Self::Regex(pattern) => pattern.is_match(input),
            Self::MatchBegining(beginning) => input.starts_with(beginning),
            Self::MatchMiddle(middle) => input.contains(middle),
            Self::MatchEnding(ending) => input.ends_with(ending)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct FileFilter {
    files: bool,
    links: bool,
    dirs: bool,
}

impl FileFilter {
    pub fn new() -> Self {
        Self {
            files: true,
            links: false,
            dirs: true,
        }
    }
    pub fn include_dir(self) -> bool {
        self.dirs
    }

    pub fn include_files(self) -> bool {
        self.files
    }

    pub fn include_links(self) -> bool {
        self.links
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    hardware_accelerate: bool,
    filter: FileFilter,
    case_insensitive: bool,
    depth: u8,
    search_type: SearchType,
    skip_errors: bool,
    max_finds: usize,
}

impl SearchOptions {
    pub fn new(key: String) -> Self {
        Self {
            filter: FileFilter::new(),
            hardware_accelerate: false,
            case_insensitive: true,
            depth: 6,
            search_type: SearchType::MatchMiddle(key),
            skip_errors: true,
            max_finds: usize::MAX,
        }
    }
}