use std::rc::Rc;

use crate::{PitouFile, PitouFileFilter};


pub struct SimplifiedSearchOptions {
    pub(crate) search_dir: Rc<PitouFile>,
    pub(crate) input: String,
    pub(crate) search_kind: u8,
    pub(crate) depth: u8,
    pub(crate) case_sensitive: bool,
    pub(crate) hardware_accelerate: bool,
    pub(crate) skip_errors: bool,
    pub(crate) filter: PitouFileFilter,
    pub(crate) max_finds: usize,
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