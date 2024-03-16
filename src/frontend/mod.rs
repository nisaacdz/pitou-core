use std::{cell::RefCell, cmp::Reverse, collections::HashSet, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{search::SimplifiedSearchOptions, PitouFile, PitouFilePath};

use self::extra::PitouFileWrapper;

pub mod msg;

pub mod extra;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum PitouFileSortOrder {
    Increasing,
    Decreasing,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum PitouFileSort {
    DateCreated(PitouFileSortOrder),
    Name(PitouFileSortOrder),
    DateModified(PitouFileSortOrder),
    DateAccessed(PitouFileSortOrder),
}

impl PitouFileSort {
    pub fn sorted(self, mut items: Vec<PitouFile>) -> Vec<PitouFile> {
        match self {
            PitouFileSort::DateCreated(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.created.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.created.datetime))
                }),
            },
            PitouFileSort::Name(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by(|a, b| a.name().cmp(&b.name()))
                }
                PitouFileSortOrder::Decreasing => {
                    items.sort_unstable_by(|a, b| b.name().cmp(&a.name()))
                }
            },
            PitouFileSort::DateModified(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.modified.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.modified.datetime))
                }),
            },
            PitouFileSort::DateAccessed(order) => match order {
                PitouFileSortOrder::Increasing => {
                    items.sort_unstable_by_key(|v| v.metadata.as_ref().map(|m| m.accessed.datetime))
                }
                PitouFileSortOrder::Decreasing => items.sort_unstable_by_key(|v| {
                    v.metadata.as_ref().map(|m| Reverse(m.accessed.datetime))
                }),
            },
        }
        items
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PitouFileFilter {
    pub files: bool,
    pub links: bool,
    pub dirs: bool,
}

impl PitouFileFilter {
    pub fn new() -> Self {
        Self {
            files: true,
            links: false,
            dirs: true,
        }
    }

    pub fn include_all() -> Self {
        Self {
            files: true,
            links: true,
            dirs: true,
        }
    }

    pub fn map(self, file: PitouFile) -> Option<PitouFile> {
        if (file.is_dir() && self.include_dirs())
            || (file.is_file() && self.include_files())
            || (file.is_link() && self.include_links())
        {
            Some(file)
        } else {
            None
        }
    }

    pub fn all_filtered(self) -> bool {
        !self.dirs && !self.files && !self.links
    }

    pub fn include_dirs(self) -> bool {
        self.dirs
    }

    pub fn include_files(self) -> bool {
        self.files
    }

    pub fn include_links(self) -> bool {
        self.links
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ItemsView {
    Grid,
    Rows,
    Tiles,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write! {f, "rgba({}, {}, {}, {})", self.0, self.1, self.2, self.3}
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ColorTheme {
    pub background1: Color,
    pub background2: Color,
    pub foreground1: Color,
    pub foreground2: Color,
    pub spare1: Color,
    pub spare2: Color,
}

impl ColorTheme {
    pub const RAMBO: Self = Self {
        background1: Color(230, 230, 230, 255),
        background2: Color(180, 180, 180, 255),
        foreground1: Color(50, 50, 50, 255),
        foreground2: Color(80, 80, 80, 255),
        spare1: Color(215, 215, 215, 255),
        spare2: Color(150, 150, 150, 255),
    };

    pub const DEFAULT_DARK: Self = Self {
        background1: Color(25, 25, 112, 255),
        background2: Color(0, 0, 51, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(192, 192, 192, 255),
        spare1: Color(153, 50, 204, 255),
        spare2: Color(255, 0, 0, 255),
    };

    pub const POLISH: Self = Self {
        background1: Color(30, 30, 30, 255),
        background2: Color(60, 60, 60, 255),
        foreground1: Color(220, 220, 220, 255),
        foreground2: Color(180, 180, 180, 255),
        spare1: Color(45, 45, 45, 255),
        spare2: Color(100, 100, 100, 255),
    };

    pub const GPT_DARK: Self = Self {
        background1: Color(50, 50, 50, 255),
        background2: Color(105, 105, 105, 255),
        foreground1: Color(240, 240, 240, 255),
        foreground2: Color(255, 255, 255, 255),
        spare1: Color(0, 0, 0, 255),
        spare2: Color(185, 210, 235, 255),
    };

    pub const OCEAN_BLUE: Self = Self {
        background1: Color(100, 200, 255, 255),
        background2: Color(50, 100, 200, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(245, 225, 180, 255),
        spare1: Color(25, 50, 100, 255),
        spare2: Color(150, 15, 50, 255),
    };

    pub const GEM_LIGHT: Self = Self {
        background1: Color(240, 240, 240, 255),
        background2: Color(200, 200, 200, 255),
        foreground1: Color(50, 50, 50, 255),
        foreground2: Color(0, 128, 128, 255),
        spare1: Color(170, 170, 170, 255),
        spare2: Color(255, 165, 0, 255),
    };

    pub const GEM_DARK: Self = Self {
        background1: Color(50, 50, 50, 255),
        background2: Color(30, 30, 30, 255),
        foreground1: Color(240, 240, 240, 255),
        foreground2: Color(0, 255, 255, 255),
        spare1: Color(100, 100, 100, 255),
        spare2: Color(255, 192, 203, 255),
    };
}

pub struct TabCtx {
    pub current_dir: RefCell<Option<Rc<PitouFile>>>,
    pub current_menu: RefCell<AppMenu>,
    pub selected_files: RefCell<HashSet<PitouFileWrapper>>,
    pub search_results: RefCell<Option<Vec<Rc<PitouFile>>>>,
    pub search_options: RefCell<Option<SimplifiedSearchOptions>>,
    pub dir_children: RefCell<Option<Rc<Vec<Rc<PitouFile>>>>>,
    pub dir_siblings: RefCell<Option<Rc<Vec<Rc<PitouFile>>>>>,
}

impl PartialEq for TabCtx {
    fn eq(&self, other: &Self) -> bool {
        self.current_dir == other.current_dir && self.current_menu == other.current_menu
    }
}

impl TabCtx {
    pub fn update_cur_dir(&self, current_dir: Option<Rc<PitouFile>>) {
        *self.current_dir.borrow_mut() = current_dir;
    }

    pub fn update_cur_menu(&self, current_menu: AppMenu) {
        *self.current_menu.borrow_mut() = current_menu;
    }

    pub fn update_selected(&self, items: HashSet<PitouFileWrapper>) {
        *self.selected_files.borrow_mut() = items;
    }

    pub fn append_selected(&self, file: Rc<PitouFile>) {
        self.selected_files
            .borrow_mut()
            .insert(PitouFileWrapper { file });
    }

    pub fn udpate_search_results(&self, results: Option<Vec<Rc<PitouFile>>>) {
        *self.search_results.borrow_mut() = results;
    }

    pub fn append_search_results(&self, items: impl Iterator<Item = Rc<PitouFile>>) {
        let mut res_borrow = self.search_results.borrow_mut();
        let res = res_borrow.get_or_insert_with(|| Vec::new());
        res.extend(items)
    }

    pub fn remove_selected(&self, file: Rc<PitouFile>) {
        self.selected_files
            .borrow_mut()
            .remove(&PitouFileWrapper { file });
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
            current_dir: RefCell::new(Some(current_dir)),
            current_menu: RefCell::new(menu),
            selected_files: RefCell::new(HashSet::new()),
            search_results: RefCell::new(None),
            dir_children: RefCell::new(None),
            dir_siblings: RefCell::new(None),
        }
    }

    pub fn default() -> Self {
        Self {
            search_options: RefCell::new(None),
            current_dir: RefCell::new(None),
            current_menu: RefCell::new(AppMenu::Home),
            selected_files: RefCell::new(HashSet::new()),
            search_results: RefCell::new(None),
            dir_children: RefCell::new(None),
            dir_siblings: RefCell::new(None),
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
            color_theme: ColorTheme::GPT_DARK,
            app_settings: AppSettings::default(),
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub refresh_rate: u8,
    pub show_extensions: bool,
    pub single_click_opens: bool,
    pub hide_impermisible: bool,
    pub show_thumbnails: bool,
    pub items_view: ItemsView,
    pub show_parents: bool,
    pub items_zoom: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            refresh_rate: 250,
            show_extensions: true,
            single_click_opens: false,
            hide_impermisible: true,
            show_thumbnails: false,
            items_view: ItemsView::Rows,
            show_parents: false,
            items_zoom: 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppMenu {
    Home,
    Explorer,
    Trash,
    Favorites,
    Search,
    Locked,
    Recents,
    Cloud,
    Settings,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GeneralFolder {
    DocumentsFolder(PitouFilePath),
    AudiosFolder(PitouFilePath),
    PicturesFolder(PitouFilePath),
    VideosFolder(PitouFilePath),
    DesktopFolder(PitouFilePath),
    DownloadsFolder(PitouFilePath),
}

impl GeneralFolder {
    pub fn name(&self) -> String {
        match self {
            GeneralFolder::DocumentsFolder(_) => String::from("Documents"),
            GeneralFolder::AudiosFolder(_) => String::from("Audios"),
            GeneralFolder::PicturesFolder(_) => String::from("pictures"),
            GeneralFolder::VideosFolder(_) => String::from("Videos"),
            GeneralFolder::DesktopFolder(_) => String::from("Desktop"),
            GeneralFolder::DownloadsFolder(_) => String::from("Downloads"),
        }
    }
}
