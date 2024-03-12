use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{frontend::PitouFileFilter, PitouFilePath};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum SearchType {
    #[serde(with = "serde_regex")]
    Regex(Regex),
    MatchBegining(String),
    MatchMiddle(String),
    MatchEnding(String),
}

impl SearchType {
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

#[derive(Serialize, Deserialize)]
pub struct SimplifiedSearchOptions {
    search_dir: PitouFilePath,
    input: String,
    search_kind: u8,
    depth: u8,
    case_sensitive: bool,
    hardware_accelerate: bool,
    skip_errors: bool,
    filter: PitouFileFilter,
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
            filter: PitouFileFilter::include_all(),
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
    pub(crate) filter: PitouFileFilter,
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
            filter: PitouFileFilter::new(),
            hardware_accelerate: false,
            case_sensitive: true,
            depth: 6,
            search_type: SearchType::MatchMiddle(key),
            skip_errors: true,
            max_finds: usize::MAX,
        }
    }
}
