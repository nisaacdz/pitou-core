use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{self, PathBuf};

use crate::{PitouFilePath, PitouFileSize};

impl Serialize for PitouFileSize {
    fn serialize<S: Serializer>(&self, sz: S) -> Result<S::Ok, S::Error> {
        sz.serialize_u64(self.bytes)
    }
}

impl<'d> Deserialize<'d> for PitouFileSize {
    fn deserialize<D: Deserializer<'d>>(dz: D) -> Result<Self, D::Error>{
        let bytes = u64::deserialize(dz)?;
        Ok(Self { bytes })
    }
}

impl Serialize for PitouFilePath {
    fn serialize<S: Serializer>(&self, sz: S) -> Result<S::Ok, S::Error> {
        serialize_pathbuf(&self.path, sz)
    }
}

impl<'d> Deserialize<'d> for PitouFilePath {
    fn deserialize<D: Deserializer<'d>>(dz: D) -> Result<Self, D::Error> {
        let path = deserialize_pathbuf(dz)?;
        Ok(Self { path })
    }
}

#[inline]
fn serialize_pathbuf<S: Serializer>(path: &PathBuf, sz: S) -> Result<S::Ok, S::Error> {
    use path::MAIN_SEPARATOR as ms;
    let path = path
        .as_os_str()
        .to_str()
        .unwrap()
        .chars()
        .map(|c| if c == ms { 28 as char } else { c })
        .collect::<String>();
    sz.collect_str(&path)
}

#[inline]
fn deserialize_pathbuf<'d, D: Deserializer<'d>>(dz: D) -> Result<PathBuf, D::Error> {
    PathBuf::deserialize(dz)
}
