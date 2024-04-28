use crate::{PitouFile, PitouFilePath, PitouFileSize};
use std::{hash::Hash, path::PathBuf};

impl From<u64> for PitouFileSize {
    fn from(bytes: u64) -> Self {
        Self { bytes }
    }
}

impl From<PathBuf> for PitouFilePath {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Hash for PitouFilePath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.path.as_os_str().as_encoded_bytes())
    }
}

impl PartialEq for PitouFile {
    fn eq(&self, other: &Self) -> bool {
        &self.path == &other.path
    }
}

impl AsRef<std::path::Path> for PitouFile {
    fn as_ref(&self) -> &std::path::Path {
        &self.path.path
    }
}
