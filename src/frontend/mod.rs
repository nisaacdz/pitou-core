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
    pub fn display_name(&self) -> String {
        let menu = *self.current_menu.borrow();
        match menu {
            AppMenu::Home => "Home".to_owned(),
            AppMenu::Explorer => {
                match &*self.folder_tracker.borrow() {
                    Some(v) => v.current().name().to_owned(),
                    None => "".to_owned(),
                }
            },
            AppMenu::Trash => "Recycle Bin".to_owned(),
            AppMenu::Favorites => "Favorites".to_owned(),
            AppMenu::Search => "Advanced Search".to_owned(),
            AppMenu::Locked => "Vault".to_owned(),
            AppMenu::Recents => "Recent Files".to_owned(),
            AppMenu::Cloud => "Cloud Storage".to_owned(),
            AppMenu::Settings => "Settings".to_owned(),
        }
    }

    pub fn reset_current_files(&self) {
        *self.dir_children.borrow_mut() = None;
        *self.dir_siblings.borrow_mut() = None;
    }

    pub fn can_navigate_backward(&self) -> bool {
        self.folder_tracker.borrow().as_ref().map(|v| v.prev().is_some()).unwrap_or(false)
    }

    pub fn can_navigate_forward(&self) -> bool {
        self.folder_tracker.borrow().as_ref().map(|v| v.next().is_some()).unwrap_or(false)
    }

    pub fn current_dir(&self) -> Option<Rc<PitouFile>> {
        self.folder_tracker.borrow().as_ref().map(|v| v.current())
    }

    pub fn navigate_backward(&self) {
        self.folder_tracker.borrow_mut().as_mut().map(|v| v.go_backward());
    }

    pub fn navigate_forward(&self) {
        self.folder_tracker.borrow_mut().as_mut().map(|v| v.go_forward());
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

pub struct GenFolderWrap {
    folder: Rc<GeneralFolder>,
}

impl GenFolderWrap {
    fn new(folder: Rc<GeneralFolder>) -> Self {
        Self { folder }
    }
}

impl PartialEq for GenFolderWrap {
    fn eq(&self, other: &Self) -> bool {
        self.folder.path() == other.folder.path()
    }
}

impl Eq for GenFolderWrap {

}

impl Hash for GenFolderWrap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.folder.path().as_bytes())
    }
}

#[derive(Clone)]
pub struct PitouFileWrap {
    inner: Rc<PitouFile>,
}

impl PitouFileWrap {
    fn new(inner: Rc<PitouFile>) -> Self {
        Self { inner }
    }
}

impl PartialEq for PitouFileWrap {
    fn eq(&self, other: &Self) -> bool {
        self.inner.path() == other.inner.path()
    }
}

impl Eq for PitouFileWrap {
    
}

impl Hash for PitouFileWrap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.inner.path().as_bytes())
    }
}

pub enum Selections {
    Drives(HashSet<PitouDriveWrap>),
    FolderEntries(FolderEntrySelections),
    SearchResults(HashSet<PitouFileWrap>),
    GeneralFolders(HashSet<GenFolderWrap>),
    RecentFiles(HashSet<PitouFileWrap>),
    PinnedFiles(HashSet<PitouFileWrap>),
    TrashItems(HashSet<PitouTrashItemWrap>),
}

pub struct PitouDriveWrap {
    drive: Rc<PitouDrive>,
}

impl PartialEq for PitouDriveWrap {
    fn eq(&self, other: &Self) -> bool {
        self.drive.mount_point() == other.drive.mount_point()
    }
}

impl Eq for PitouDriveWrap {

}

impl PitouDriveWrap {
    fn new(drive: Rc<PitouDrive>) -> Self {
        Self {
            drive
        }
    }
}

impl Hash for PitouDriveWrap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.drive.mount_point().as_bytes())
    }
}

#[derive(Clone)]
pub struct PitouTrashItemWrap {
    item: Rc<PitouTrashItem>,
}

impl PitouTrashItemWrap {
    fn new(item: Rc<PitouTrashItem>) -> Self {
        Self {
            item
        }
    }
}

impl PartialEq for PitouTrashItemWrap {
    fn eq(&self, other: &Self) -> bool {
        self.item.id() == other.item.id()
    }
}

impl Eq for PitouTrashItemWrap {

}

impl Hash for PitouTrashItemWrap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.item.id().as_bytes())
    }
}

struct FolderEntry {
    item: Rc<PitouFile>
}

impl FolderEntry {
    fn new(item: Rc<PitouFile>) -> Self {
        Self { item }
    }
}

impl PartialEq for FolderEntry  {
    fn eq(&self, other: &Self) -> bool {
        self.item.name() == other.item.name()
    }
}

impl Eq for FolderEntry {
    
}

impl Hash for FolderEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.item.name().as_bytes())
    }
}

pub struct FolderEntrySelections {
    items: HashSet<FolderEntry>,
}

pub struct StaticData {
    pub drives: RefCell<Option<Rc<Vec<Rc<PitouDrive>>>>>,
    pub selections: RefCell<Selections>,
    pub trash_items: RefCell<Option<Rc<Vec<Rc<PitouTrashItem>>>>>,
    pub gen_dirs: RefCell<Option<Rc<Vec<Rc<GeneralFolder>>>>>,
}

impl StaticData {
    pub fn new() -> Self {
        Self {
            drives: RefCell::new(None),
            selections: RefCell::new(Selections::Drives(HashSet::new())),
            trash_items: RefCell::new(None),
            gen_dirs: RefCell::new(None)
        }
    }

    pub fn can_attempt_delete(&self) -> bool {
        match &*self.selections.borrow() {
            Selections::Drives(_) => false,
            Selections::FolderEntries(fe) => fe.items.len() > 0,
            Selections::SearchResults(sr) => sr.len() > 0,
            Selections::GeneralFolders(_) => false,
            Selections::RecentFiles(rf) => rf.len() > 0,
            Selections::PinnedFiles(pf) => pf.len() > 0,
            Selections::TrashItems(ti) => ti.len() > 0,
        }
    }

    pub fn openable_selection(&self) -> Option<Rc<PitouFile>> {
        match &*self.selections.borrow() {
            Selections::Drives(_) => None,
            Selections::FolderEntries(fe) => {
                fe.items.iter().next().map(|v| v.item.clone())
            },
            Selections::SearchResults(sr) => {
                sr.iter().next().map(|v| v.inner.clone())
            },
            Selections::GeneralFolders(_) => todo!(),
            Selections::RecentFiles(rf) => {
                rf.iter().next().map(|v| v.inner.clone())
            },
            Selections::PinnedFiles(pf) => {
                pf.iter().next().map(|v| v.inner.clone())
            },
            Selections::TrashItems(_) => None,
        }
    }
    
    pub fn select_drive(&self, drive: Rc<PitouDrive>) {
        let mut selections = self.selections.borrow_mut();
        if let Selections::Drives(d) = &mut *selections {
            d.insert(PitouDriveWrap::new(drive));
        } else {
            let new_set = HashSet::from_iter(Some(PitouDriveWrap::new(drive)));
            *selections = Selections::Drives(new_set)
        }
    }

    pub fn folder_entry_selections(&self) -> Option<Vec<Rc<PitouFile>>> {
        if let Selections::FolderEntries(fe) = &*self.selections.borrow() {
            Some(fe.items.iter().map(|v| v.item.clone()).collect())
        } else {
            None
        }
    }

    pub fn select_folder_entry(&self, item: Rc<PitouFile>) {
        let mut selections = self.selections.borrow_mut();
        if let Selections::FolderEntries(fe) = &mut *selections {
            fe.items.insert(FolderEntry::new(item));
        } else {
            let items = HashSet::from_iter(Some(FolderEntry::new(item)));
            *selections = Selections::FolderEntries(FolderEntrySelections { items })
        }
    }

    pub fn select_search_result(&self, item: Rc<PitouFile>) {
        let mut selections = self.selections.borrow_mut();
        if let Selections::SearchResults(sr) = &mut *selections {
            sr.insert(PitouFileWrap::new(item));
        } else {
            let new_set = HashSet::from_iter(Some(PitouFileWrap::new(item)));
            *selections = Selections::SearchResults(new_set)
        }
    }

    pub fn select_gen_folder(&self, folder: Rc<GeneralFolder>) {
        let mut selections = self.selections.borrow_mut();
        if let Selections::GeneralFolders(gf) = &mut *selections {
            gf.insert(GenFolderWrap::new(folder));
        } else {
            let new_set = HashSet::from_iter(Some(GenFolderWrap::new(folder)));
            *selections = Selections::GeneralFolders(new_set)
        }
    }

    pub fn select_trash_item(&self, item: Rc<PitouTrashItem>) {
        let mut selections = self.selections.borrow_mut();
        if let Selections::TrashItems(vals) = &mut *selections {
            vals.insert(PitouTrashItemWrap::new(item));
        } else {
            let new_set = HashSet::from_iter(Some(PitouTrashItemWrap::new(item)));
            *selections = Selections::TrashItems(new_set)
        }
    }

    pub fn update_trash_items(&self, items: Option<Rc<Vec<Rc<PitouTrashItem>>>>) {
        *self.trash_items.borrow_mut() = items;
    }

    pub fn trash_items(&self) -> Option<Rc<Vec<Rc<PitouTrashItem>>>> {
        (*self.trash_items.borrow()).clone()
    }

    pub fn reset_trash_items(&self) {
        *self.trash_items.borrow_mut() = None;
    }

    pub fn no_trash_items(&self) -> bool {
        self.trash_items.borrow().is_none()
    }

    pub fn reset_drives(&self) {
        *self.drives.borrow_mut() = None;
    }

    pub fn reset_gen_dirs(&self) {
        *self.gen_dirs.borrow_mut() = None;
    }

    pub fn update_gen_dirs(&self, dirs: Option<Rc<Vec<Rc<GeneralFolder>>>>) {
        *self.gen_dirs.borrow_mut() = dirs;
    }

    pub fn gen_dirs(&self) -> Option<Rc<Vec<Rc<GeneralFolder>>>>{
        (&*self.gen_dirs.borrow()).clone()
    }

    pub fn update_drives(&self, drives: Rc<Vec<Rc<PitouDrive>>>) {
        *self.drives.borrow_mut() = Some(drives);
    }

    pub fn clear_dir_entry_selection(&self, item: Rc<PitouFile>) {
        if let Selections::FolderEntries(en) = &mut *self.selections.borrow_mut() {
            en.items.remove(&FolderEntry { item });
        }
    }

    pub fn clear_drive_selection(&self, drive: Rc<PitouDrive>) {
        if let Selections::Drives(d) = &mut *self.selections.borrow_mut() {
            d.remove(&PitouDriveWrap { drive });
        }
    }

    pub fn clear_gen_folder_selection(&self, folder: Rc<GeneralFolder>) {
        if let Selections::GeneralFolders(gf) = &mut *self.selections.borrow_mut() {
            gf.remove(&GenFolderWrap { folder });
        }
    }

    pub fn clear_trash_item_selection(&self, item: Rc<PitouTrashItem>) {
        if let Selections::TrashItems(ti) = &mut *self.selections.borrow_mut() {
            ti.remove(&PitouTrashItemWrap { item });
        }
    }

    pub fn clear_all_selections(&self) {
        *self.selections.borrow_mut() = Selections::FolderEntries(FolderEntrySelections { items: HashSet::new() })
    }

    pub fn is_selected_dir_entry(&self, item: Rc<PitouFile>) -> bool {
        if let Selections::FolderEntries(en) = &*self.selections.borrow() {
            en.items.contains(&FolderEntry { item })
        } else {
            false
        }
    }

    pub fn is_selected_drive(&self, drive: Rc<PitouDrive>) -> bool {
        if let Selections::Drives(d) = &*self.selections.borrow() {
            d.contains(&PitouDriveWrap { drive })
        } else {
            false
        }
    }

    pub fn is_selected_gen_folder(&self, folder: Rc<GeneralFolder>) -> bool {
        if let Selections::GeneralFolders(gf) = &*self.selections.borrow() {
            gf.contains(&GenFolderWrap { folder })
        } else {
            false
        }
    }

    pub fn is_selected_trash_item(&self, item: Rc<PitouTrashItem>) -> bool {
        if let Selections::TrashItems(ti) = &*self.selections.borrow() {
            ti.contains(&PitouTrashItemWrap { item })
        } else {
            false
        }
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
            color_theme: ColorTheme::DEFAULT_DARK,
            app_settings: AppSettings::default(),
        }
    }
}

#[derive(Clone)]
pub struct AllTabsCtx {
    pub all_tabs: Rc<RefCell<Vec<Rc<TabCtx>>>>,
    pub active_tab: usize,
}


impl AllTabsCtx {
    pub fn default() -> Self {
        let active_tab = Rc::new(TabCtx::default());
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

#[derive(Clone, Copy, PartialEq)]
enum State {
    State1, State2
}

#[derive(PartialEq)]
pub struct RefresherState {
    state: RefCell<State>,
}

impl RefresherState {
    pub fn default() -> Self {
        Self { state: RefCell::new(State::State1) }
    }
}


#[derive(Clone)]
pub struct ApplicationContext {
    pub gen_ctx: Rc<RefCell<GenCtx>>,
    pub active_tab: Rc<TabCtx>,
    pub static_data: Rc<StaticData>,
    pub refresher_state: Rc<RefresherState>,
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
            refresher_state: Rc::new(RefresherState::default())
        }
    }

    pub fn refresher_state(&self) -> Rc<RefresherState> {
        self.refresher_state.clone()
    }

    pub fn toggle_refresher_state(&self) {
        let mut state = self.refresher_state.state.borrow_mut();
        match *state {
            State::State1 => *state = State::State2,
            State::State2 => *state = State::State1,
        }
    }

    pub fn current_menu(&self) -> AppMenu {
        *self.active_tab.current_menu.borrow()
    }

    pub fn color_theme(&self) -> ColorTheme {
        self.gen_ctx.borrow().color_theme
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
