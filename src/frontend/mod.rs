use std::{
    cell::RefCell,
    collections::HashSet,
    hash::{Hash, Hasher},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    search::SimplifiedSearchOptions, AppMenu, AppSettings, ColorTheme, GeneralFolder, PitouDrive,
    PitouFile, PitouTrashItem,
};

use self::extra::FolderTracker;

pub mod extra;

pub struct TabCtx {
    pub folder_tracker: RefCell<Option<FolderTracker>>,
    pub current_menu: RefCell<AppMenu>,
    pub search_results: RefCell<Option<Vec<Rc<PitouFile>>>>,
    pub search_options: RefCell<Option<SimplifiedSearchOptions>>,
    pub dir_children: RefCell<Option<Rc<Vec<Rc<PitouFile>>>>>,
    pub dir_siblings: RefCell<Option<Rc<Vec<Rc<PitouFile>>>>>,
}

impl TabCtx {
    pub fn current_dir(&self) -> Option<Rc<PitouFile>> {
        self.folder_tracker.borrow().as_ref().map(|v| v.current())
    }

    pub fn update_cur_dir(&self, current_dir: Option<Rc<PitouFile>>) {
        if let Some(current_dir) = current_dir {
            let mut borrow = self.folder_tracker.borrow_mut();
            if let Some(val) = &mut *borrow {
                val.update_directory(current_dir)
            } else {
                let _ = borrow.insert(FolderTracker::new(current_dir));
            }
        } else {
            self.folder_tracker.borrow_mut().take();
        }
    }

    pub fn update_cur_menu(&self, current_menu: AppMenu) {
        *self.current_menu.borrow_mut() = current_menu;
    }

    pub fn udpate_search_results(&self, results: Option<Vec<Rc<PitouFile>>>) {
        *self.search_results.borrow_mut() = results;
    }

    pub fn append_search_results(&self, items: impl Iterator<Item = Rc<PitouFile>>) {
        let mut res_borrow = self.search_results.borrow_mut();
        let res = res_borrow.get_or_insert_with(|| Vec::new());
        res.extend(items)
    }

    pub fn update_children(&self, children: Option<Rc<Vec<Rc<PitouFile>>>>) {
        *self.dir_children.borrow_mut() = children;
    }

    pub fn update_siblings(&self, siblings: Option<Rc<Vec<Rc<PitouFile>>>>) {
        *self.dir_siblings.borrow_mut() = siblings;
    }

    pub fn update_search_options(&self, search_options: Option<SimplifiedSearchOptions>) {
        *self.search_options.borrow_mut() = search_options;
    }

    pub fn new_with_dir(current_dir: Rc<PitouFile>, menu: AppMenu) -> Self {
        Self {
            search_options: RefCell::new(Some(SimplifiedSearchOptions::default(
                current_dir.clone(),
            ))),
            folder_tracker: RefCell::new(Some(FolderTracker::new(current_dir))),
            current_menu: RefCell::new(menu),
            search_results: RefCell::new(None),
            dir_children: RefCell::new(None),
            dir_siblings: RefCell::new(None),
        }
    }

    pub fn default() -> Self {
        Self {
            search_options: RefCell::new(None),
            folder_tracker: RefCell::new(None),
            current_menu: RefCell::new(AppMenu::Home),
            search_results: RefCell::new(None),
            dir_children: RefCell::new(None),
            dir_siblings: RefCell::new(None),
        }
    }
}

pub struct StaticData {
    pub drives: RefCell<Rc<Vec<Rc<PitouDrive>>>>,
    pub selections: RefCell<HashSet<VWrapper>>,
}

impl StaticData {
    pub fn new() -> Self {
        Self {
            drives: RefCell::new(Rc::new(Vec::new())),
            selections: RefCell::new(HashSet::new()),
        }
    }

    pub fn update_drives(&self, drives: Rc<Vec<Rc<PitouDrive>>>) {
        *self.drives.borrow_mut() = drives;
    }

    pub fn clear_selection(&self, item: VWrapper) {
        self.selections.borrow_mut().remove(&item);
    }

    pub fn clear_all_selections(&self) {
        self.selections.borrow_mut().clear()
    }

    pub fn is_selected(&self, item: VWrapper) -> bool {
        self.selections.borrow().contains(&item)
    }

    pub fn add_selection(&self, item: VWrapper) {
        self.selections.borrow_mut().insert(item);
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct GenCtx {
    pub app_width: i32,
    pub app_height: i32,
    pub color_theme: ColorTheme,
    pub app_settings: AppSettings,
}

impl Default for GenCtx {
    fn default() -> Self {
        Self {
            app_width: 1366,
            app_height: 768,
            color_theme: ColorTheme::GPT_DARK,
            app_settings: AppSettings::default(),
        }
    }
}

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
