use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::PitouFilePath;
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
}

#[test]
fn test_ignore_case_functions() {
    let input = "zXcVbNm<>?";
    let key = "CvbnM<>?";
    assert!(SearchType::ends_with_ignore_case(key, input))
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct FileFilter {
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
pub struct SearchOptions {
    pub(crate) search_dir: PitouFilePath,
    pub(crate) hardware_accelerate: bool,
    pub(crate) filter: FileFilter,
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
            filter: FileFilter::new(),
            hardware_accelerate: false,
            case_sensitive: true,
            depth: 6,
            search_type: SearchType::MatchMiddle(key),
            skip_errors: true,
            max_finds: usize::MAX,
        }
    }
}
