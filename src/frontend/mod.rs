use std::{cell::RefCell, collections::HashSet, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{search::SimplifiedSearchOptions, PitouFile, PitouFilePath};

pub(crate) mod msg;

#[derive(Serialize, Deserialize)]
pub enum ItemsView {
    Grid,
    Rows,
    Tiles,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Color(pub u8,pub u8,pub u8,pub u8);

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!{f, "rgba({}, {}, {}, {})", self.0, self.1, self.2, self.3}
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ColorTheme {
    pub background1: Color,
    pub background2: Color,
    pub foreground1: Color,
    pub foreground2: Color,
    pub spare1: Color,
    pub spare2: Color,
}

impl ColorTheme {
    pub const DEFAULT_LIGHT: Self = Self {
        background1: Color(204, 229, 255, 255),
        background2: Color(220, 220, 220, 255),
        foreground1: Color(0, 0, 0, 255),
        foreground2: Color(128, 128, 128, 255),
        spare1: Color(255, 127, 80, 255),
        spare2: Color(0, 128, 0, 255),
    };

    pub const DEFAULT_DARK: Self = Self {
        background1: Color(25, 25, 112, 255),
        background2: Color(0, 0, 51, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(192, 192, 192, 255),
        spare1: Color(153, 50, 204, 255),
        spare2: Color(255, 0, 0, 255),
    };

    pub const GPT_LIGHT: Self = Self {
        background1: Color(240, 240, 240, 255),
        background2: Color( 255, 255, 255, 255),
        foreground1: Color(80, 80, 80, 255),
        foreground2: Color(0, 0, 0, 255),
        spare1: Color(173, 216, 230, 255),
        spare2: Color(144, 238, 144, 255),
    };

    pub const GPT_DARK: Self = Self {
        background1: Color(50, 50, 50, 255),
        background2: Color(105, 105, 105, 255),
        foreground1: Color(240, 240, 240, 255),
        foreground2: Color( 255, 255, 255, 255),
        spare1: Color(65, 105, 225, 255),
        spare2: Color(0, 128, 0, 255),
    };

    pub const OCEAN_BLUE: Self = Self {
        background1: Color(100, 150, 250, 255),
        background2: Color(50, 100, 200, 255),
        foreground1: Color(255, 255, 255, 255),
        foreground2: Color(245, 225, 180, 255),
        spare1: Color(25, 50, 100, 255),
        spare2: Color(150, 15, 50, 255),
    };
}

pub struct TabCtx {
    pub current_dir: PitouFilePath,
    pub current_menu: AppMenu,
    pub selected_files: Rc<RefCell<HashSet<PitouFile>>>,
    pub search_results: Option<Rc<Vec<PitouFile>>>,
    pub search_options: SimplifiedSearchOptions,
    pub dir_children: Option<Rc<Vec<PitouFile>>>,
    pub dir_siblings: Option<Rc<Vec<PitouFile>>>,
}

impl PartialEq for TabCtx {
    fn eq(&self, other: &Self) -> bool {
        // Later to change this to false
        self.current_dir == other.current_dir && self.current_menu == other.current_menu
    }
}

impl TabCtx {
    pub fn default() -> Self {
        TabCtx::new(
            PitouFilePath {
                path: std::path::PathBuf::from("C:/Users/nisaacdz"),
            },
            AppMenu::Explorer,
        )
    }
    pub fn new(current_dir: PitouFilePath, current_menu: AppMenu) -> Self {
        Self {
            search_options: SimplifiedSearchOptions::default(current_dir.path.clone().into()),
            current_dir,
            current_menu,
            selected_files: Rc::new(RefCell::new(HashSet::new())),
            search_results: None,
            dir_children: None,
            dir_siblings: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
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
            color_theme: ColorTheme::OCEAN_BLUE,
            app_settings: AppSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub refresh_rate: u8,
    pub show_extensions: bool,
    pub single_click_opens: bool,
    pub hide_impermisible: bool,
    pub show_thumbnails: bool,
    pub items_view: ItemsView,
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
            items_zoom: 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppMenu {
    Home,
    Explorer,
    Trash,
    Bookmarks,
    Recents,
    Cloud,
    Settings,
}

pub struct Width {
    pub value: i32,
}

impl From<i32> for Width {
    fn from(value: i32) -> Self {
        Width { value }
    }
}

pub struct Height {
    pub value: i32,
}

impl From<i32> for Height {
    fn from(value: i32) -> Self {
        Height { value }
    }
}

impl std::fmt::Display for Width {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "width: {}px;", self.value)
    }
}

impl std::fmt::Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "height: {}px;", self.value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    #[allow(unused)]
    pub fn width(self) -> Width {
        Width { value: self.width }
    }

    pub fn height(self) -> Height {
        Height { value: self.height }
    }
}

impl std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "width: {}px;\nheight: {}px;", self.width, self.height)
    }
}
