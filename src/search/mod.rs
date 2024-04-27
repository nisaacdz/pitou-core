use std::rc::Rc;

use crate::{PitouFile, PitouFileFilter};

pub struct SimplifiedSearchOptions {
    pub search_dir: Rc<PitouFile>,
    pub input: String,
    pub search_kind: u8,
    pub depth: u8,
    pub case_sensitive: bool,
    pub hardware_accelerate: bool,
    pub skip_errors: bool,
    pub filter: PitouFileFilter,
    pub max_finds: usize,
}

impl SimplifiedSearchOptions {
    pub fn default(current_dir: Rc<PitouFile>) -> Self {
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
}
