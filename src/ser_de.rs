use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};
use std::path::{self, PathBuf};

use crate::{frontend::{AppMenu, TabCtx}, PitouFilePath, PitouFileSize};

impl Serialize for TabCtx {
    fn serialize<S: Serializer>(&self, sz: S) -> Result<S::Ok, S::Error> {
        let mut ss = sz.serialize_struct("TabCtx", 2)?;
        ss.serialize_field("current_dir", &self.current_dir)?;
        ss.serialize_field("current_menu", &self.current_menu)?;
        ss.end()
    }
}

impl<'d> Deserialize<'d> for TabCtx {
    fn deserialize<D: Deserializer<'d>>(dz: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        struct TempVal {
            current_dir: PitouFilePath,
            current_menu: AppMenu,
        }

        let TempVal { current_dir, current_menu } = TempVal::deserialize(dz)?;
        
        Ok(TabCtx::new(current_dir, current_menu))
    }
}

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