use std::{hash::Hash, rc::Rc};

use crate::PitouFile;

pub struct PitouFileWrapper {
    pub file: Rc<PitouFile>,
}

impl PartialEq for PitouFileWrapper {
    fn eq(&self, other: &Self) -> bool {
        &*self.file == &*other.file
    }
}

impl Eq for PitouFileWrapper {}

impl Hash for PitouFileWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file.path.hash(state)
    }
}
