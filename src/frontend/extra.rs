use std::{hash::{Hash, Hasher}, rc::Rc};

use crate::{frontend::GeneralFolder, PitouDrive, PitouFile, PitouTrashItem};

pub enum VWrapper {
    Drive(Rc<PitouDrive>),
    GenFolder(Rc<GeneralFolder>),
    FirstAncestor(Rc<PitouFile>),
    FullPath(Rc<PitouFile>),
    TrashItem(Rc<PitouTrashItem>),
}

impl Hash for VWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = match self {
            VWrapper::Drive(d) => d.mount_point.as_bytes(),
            VWrapper::GenFolder(g) => g.o_name().as_bytes(),
            VWrapper::FirstAncestor(f) => f.name().as_bytes(),
            VWrapper::FullPath(f) => f.path.as_bytes(),
            VWrapper::TrashItem(t) => t.metadata.id.as_bytes(),
        };
        state.write(bytes);
    }
}

impl PartialEq for VWrapper {
    fn eq(&self, other: &Self) -> bool {
        match self {
            VWrapper::Drive(d1) => matches!(other, Self::Drive(d2) if d1 == d2),
            VWrapper::GenFolder(g1) => {
                matches!(other, Self::GenFolder(g2) if g1.o_name() == g2.o_name())
            }
            VWrapper::FirstAncestor(a1) => {
                matches!(other, Self::FirstAncestor(a2) if a1.name() == a2.name())
            }
            VWrapper::FullPath(f1) => matches!(other, Self::FullPath(f2) if f1.path == f2.path),
            VWrapper::TrashItem(t1) => {
                matches!(other, Self::TrashItem(t2) if t1.original_path == t2.original_path)
            }
        }
    }
}

impl Eq for VWrapper {}