use serde::{de::{SeqAccess, Visitor}, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::path::{self, PathBuf};

use crate::{
    frontend::{AppMenu, TabCtx},
    search::SimplifiedSearchOptions,
    PitouFile, PitouFilePath, PitouFileSize,
};

impl Serialize for TabCtx {
    fn serialize<S: Serializer>(&self, sz: S) -> Result<S::Ok, S::Error> {
        let mut ss = sz.serialize_struct("TabCtx", 6)?;
        ss.serialize_field("current_dir", &*self.current_dir)?;
        ss.serialize_field("current_menu", &self.current_menu)?;
        ss.serialize_field("selected_files", &Vec::<PitouFile>::new())?;
        ss.serialize_field("search_results", &None::<Option<Vec<PitouFile>>>)?;
        ss.serialize_field("dir_children", &None::<Option<Vec<PitouFile>>>)?;
        ss.serialize_field("dir_siblings", &None::<Option<Vec<PitouFile>>>)?;
        ss.end()
    }
}

impl<'d> Deserialize<'d> for TabCtx {
    fn deserialize<D: Deserializer<'d>>(dz: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        struct TempVal {
            pub current_dir: PitouFilePath,
            pub current_menu: AppMenu,
            pub selected_files: Vec<PitouFile>,
            pub search_results: Option<Vec<PitouFile>>,
            pub dir_children: Option<Vec<PitouFile>>,
            pub dir_siblings: Option<Vec<PitouFile>>,
        }

        let TempVal {
            current_dir,
            current_menu,
            selected_files: _,
            search_results: _,
            dir_children: _,
            dir_siblings: _,
        } = TempVal::deserialize(dz)?;

        Ok(TabCtx::new(current_dir, current_menu))
    }
}

impl Serialize for PitouFileSize {
    fn serialize<S: Serializer>(&self, sz: S) -> Result<S::Ok, S::Error> {
        sz.serialize_u64(self.bytes)
    }
}

impl<'d> Deserialize<'d> for PitouFileSize {
    fn deserialize<D: Deserializer<'d>>(dz: D) -> Result<Self, D::Error> {
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
    use path::MAIN_SEPARATOR as ms;
    let mut res = String::deserialize(dz)?;
    for bc in unsafe { res.as_bytes_mut() } {
        if *bc == 28 {
            *bc = ms as u8;
        }
    }
    Ok(PathBuf::from(res))
}
