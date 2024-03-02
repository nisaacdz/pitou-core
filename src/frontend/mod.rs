use std::{cell::RefCell, collections::HashSet, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{search::SimplifiedSearchOptions, PitouFile, PitouFilePath};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Color(u8, u8, u8, u8);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ColorTheme {
    background1: Color,
    background2: Color,
    foreground1: Color,
    foreground2: Color,
    spare1: Color,
    spare2: Color,
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

impl TabCtx {
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
    app_width: i32,
    app_height: i32,
    color_theme: ColorTheme,
    app_settings: AppSettings,
}

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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