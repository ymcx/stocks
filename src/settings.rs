use crate::APP_ID;
use adw::gtk::gio::{self, prelude::*};

#[derive(Clone)]
pub struct Settings {
    settings: gio::Settings,
}

impl Settings {
    pub fn new() -> Self {
        let settings = gio::Settings::new(APP_ID);
        Self { settings }
    }

    pub fn set_window_width(&self, width: i32) {
        self.settings.set_int("window-width", width).unwrap();
    }

    pub fn set_window_height(&self, height: i32) {
        self.settings.set_int("window-height", height).unwrap();
    }

    pub fn set_window_maximized(&self, maximized: bool) {
        self.settings
            .set_boolean("window-maximized", maximized)
            .unwrap();
    }

    pub fn set_last_stock(&self, stock: &str) {
        self.settings
            .set_string("last-stock", stock)
            .unwrap();
    }

    pub fn set_last_range(&self, range: &str) {
        self.settings
            .set_string("last-range", range)
            .unwrap();
    }

    pub fn get_window_width(&self) -> i32 {
        self.settings.int("window-width")
    }

    pub fn get_window_height(&self) -> i32 {
        self.settings.int("window-height")
    }

    pub fn get_window_maximized(&self) -> bool {
        self.settings.boolean("window-maximized")
    }

    pub fn get_last_stock(&self) -> String {
        self.settings.string("last-stock").to_string()
    }

    pub fn get_last_range(&self) -> String {
        self.settings.string("last-range").to_string()
    }

    pub fn get_bookmarks(&self) -> Vec<String> {
        self.settings
            .strv("bookmarks")
            .into_iter()
            .map(|i| i.to_string())
            .collect()
    }

    pub fn find_bookmark(&self, bookmark: &str) -> Option<usize> {
        let bookmarks = self.get_bookmarks();
        bookmarks.into_iter().position(|i| i == bookmark)
    }

    pub fn set_bookmarks(&self, bookmarks: Vec<String>) {
        self.settings.set_strv("bookmarks", bookmarks).unwrap();
    }

    pub fn add_bookmark(&self, bookmark: &str) {
        let mut bookmarks = self.get_bookmarks();
        bookmarks.push(bookmark.to_string());
        self.set_bookmarks(bookmarks);
    }

    pub fn remove_bookmark(&self, index: usize) {
        let mut bookmarks = self.get_bookmarks();
        bookmarks.remove(index as usize);
        self.set_bookmarks(bookmarks);
    }

    pub fn reorder_bookmarks(&self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }

        let mut bookmarks = self.get_bookmarks();
        let temp = bookmarks[index1].to_string();
        if index1 < index2 {
            for i in index1 + 1..=index2 {
                bookmarks[i - 1] = bookmarks[i].to_string();
            }
        } else {
            for i in (index2 + 1..=index1).rev() {
                bookmarks[i] = bookmarks[i - 1].to_string();
            }
        }
        bookmarks[index2] = temp;
        self.set_bookmarks(bookmarks);
    }
}
