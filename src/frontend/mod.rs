use std::{
    cell::RefCell,
    collections::HashSet,
    hash::{Hash, Hasher},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{search::SimplifiedSearchOptions, AppMenu, AppSettings, ColorTheme, GeneralFolder, ItemsView, PitouDrive, PitouFile, PitouTrashItem};

use self::extra::FolderTracker;
pub mod ser_de;

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

#[derive(Clone)]
pub struct AllTabsCtx {
    pub all_tabs: Rc<RefCell<Vec<Rc<TabCtx>>>>,
    pub active_tab: usize,
}


impl AllTabsCtx {
    pub fn default() -> Self {
        let active_tab = Rc::new(TabCtx::default());
        active_tab.update_cur_menu(AppMenu::Explorer);
        let all_tabs = Rc::new(RefCell::new(vec![active_tab]));
        Self {
            all_tabs,
            active_tab: 0,
        }
    }

    pub fn add_tab(mut self) -> Self {
        let mut all_tabs = self.all_tabs.borrow_mut();
        let next_idx = all_tabs.len();
        all_tabs.push(Rc::new(TabCtx::default()));
        std::mem::drop(all_tabs);
        self.active_tab = next_idx;
        self
    }

    pub fn change_tab(mut self, idx: usize) -> Self {
        self.active_tab = idx;
        self
    }

    pub fn remove_tab(mut self, idx: usize) -> Option<Self> {
        let mut all_tabs = self.all_tabs.borrow_mut();
        if all_tabs.len() <= 1 {
            return None;
        }
        all_tabs.remove(idx);
        std::mem::drop(all_tabs);
        if idx <= self.active_tab {
            if self.active_tab != 0 {
                self.active_tab -= 1;
            }
        }
        Some(self)
    }

    pub fn current_tab(&self) -> Rc<TabCtx> {
        self.all_tabs.borrow()[self.active_tab].clone()
    }

    pub fn change_menu(self, menu: AppMenu) -> Self {
        let current_tab = self.current_tab();
        *current_tab.current_menu.borrow_mut() = menu;
        self
    }
}


#[derive(Clone)]
pub struct ApplicationContext {
    pub gen_ctx: Rc<RefCell<GenCtx>>,
    pub active_tab: Rc<TabCtx>,
    pub static_data: Rc<StaticData>,
}

impl PartialEq for ApplicationContext {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl ApplicationContext {
    pub fn new(gen_ctx: Rc<RefCell<GenCtx>>, active_tab: Rc<TabCtx>, static_data: Rc<StaticData>) -> Self {
        Self {
            gen_ctx,
            active_tab,
            static_data,
        }
    }

    pub fn update_color_theme(&self, new_theme: ColorTheme) {
        self.gen_ctx.borrow_mut().color_theme = new_theme;
    }

    pub fn update_refresh_rate(&self, new_rate: u8) {
        self.gen_ctx.borrow_mut().app_settings.refresh_rate = new_rate;
    }

    pub fn toggle_show_extensions(&self, new_val: bool) {
        self.gen_ctx.borrow_mut().app_settings.show_extensions = new_val;
    }

    pub fn toggle_hide_system_files(&self, new_val: bool) {
        self.gen_ctx.borrow_mut().app_settings.hide_system_files = new_val;
    }

    pub fn toggle_show_thumbnails(&self, new_val: bool) {
        self.gen_ctx.borrow_mut().app_settings.show_thumbnails = new_val;
    }

    pub fn toggle_show_parents(&self, new_val: bool) {
        self.gen_ctx.borrow_mut().app_settings.show_parents = new_val;
    }

    pub fn update_items_view(&self, new_view: ItemsView) {
        self.gen_ctx.borrow_mut().app_settings.items_view = new_view;
    }

    pub fn update_zoom_value(&self, new_val: f32) {
        self.gen_ctx.borrow_mut().app_settings.items_zoom = new_val;
    }

    pub fn hide_system_files(&self) -> bool {
        self.gen_ctx.borrow().app_settings.hide_system_files
    }

    pub fn refresh_rate(&self) -> u8 {
        self.gen_ctx.borrow().app_settings.refresh_rate
    }

    pub fn show_thumbnails(&self) -> bool {
        self.gen_ctx.borrow().app_settings.show_thumbnails
    }

    pub fn items_view(&self) -> ItemsView {
        self.gen_ctx.borrow().app_settings.items_view
    }

    pub fn items_zoom(&self) -> f32 {
        self.gen_ctx.borrow().app_settings.items_zoom
    }

    pub fn show_parents(&self) -> bool {
        self.gen_ctx.borrow().app_settings.show_parents
    }

    pub fn show_extensions(&self) -> bool {
        self.gen_ctx.borrow().app_settings.show_extensions
    }
}
