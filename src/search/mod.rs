use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::PitouFilePath;
#[cfg(feature = "backend")]
pub mod ops;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum SearchType {
    #[serde(with = "serde_regex")]
    Regex(Regex),
    MatchBegining(String),
    MatchMiddle(String),
    MatchEnding(String),
}

impl SearchType {
    fn matches(&self, input: &str, sensitive: bool) -> bool {
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

    fn parse_regex(search_kind: u8, search_key: String) -> Option<Self> {
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

#[test]
fn test_ignore_case_functions() {
    let input = "zXcVbNm<>?";
    let key = "CvbnM<>?";
    assert!(SearchType::ends_with_ignore_case(key, input))
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct SearchFilter {
    files: bool,
    links: bool,
    dirs: bool,
}

impl SearchFilter {
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

#[derive(Serialize, Deserialize)]
pub struct SimplifiedSearchOptions {
    search_dir: PitouFilePath,
    input: String,
    search_kind: u8,
    depth: u8,
    case_sensitive: bool,
    hardware_accelerate: bool,
    skip_errors: bool,
    filter: SearchFilter,
    max_finds: usize,
}

impl SimplifiedSearchOptions {
    pub fn default(current_dir: PitouFilePath) -> Self {
        Self {
            search_dir: current_dir,
            input: String::new(),
            search_kind: 1,
            depth: 6,
            case_sensitive: true,
            hardware_accelerate: false,
            skip_errors: true,
            filter: SearchFilter::include_all(),
            max_finds: 250,
        }
    }

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

#[derive(Serialize, Deserialize)]
pub struct SearchOptions {
    pub(crate) search_dir: PitouFilePath,
    pub(crate) hardware_accelerate: bool,
    pub(crate) filter: SearchFilter,
    pub(crate) case_sensitive: bool,
    pub(crate) depth: u8,
    pub(crate) search_type: SearchType,
    pub(crate) skip_errors: bool,
    pub(crate) max_finds: usize,
}

impl SearchOptions {
    pub fn new(search_dir: PitouFilePath, key: String) -> Self {
        Self {
            search_dir,
            filter: SearchFilter::new(),
            hardware_accelerate: false,
            case_sensitive: true,
            depth: 6,
            search_type: SearchType::MatchMiddle(key),
            skip_errors: true,
            max_finds: usize::MAX,
        }
    }
}
