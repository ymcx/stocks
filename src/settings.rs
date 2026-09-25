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

    pub fn get_window_width(&self) -> i32 {
        self.settings.int("window-width")
    }

    pub fn get_window_height(&self) -> i32 {
        self.settings.int("window-height")
    }

    pub fn get_window_maximized(&self) -> bool {
        self.settings.boolean("window-maximized")
    }

    pub fn get_bookmarks(&self) -> Vec<String> {
        self.settings
            .strv("bookmarks")
            .iter()
            .map(|i| i.to_string())
            .collect()
    }

    pub fn set_bookmarks(&self, bookmarks: Vec<String>) {
        self.settings.set_strv("bookmarks", bookmarks).unwrap();
    }

    pub fn add_bookmarks(&self, bookmark: &str) {
        let mut bookmarks = self.get_bookmarks();
        bookmarks.push(bookmark.to_string());
        self.set_bookmarks(bookmarks);
    }

    pub fn del_bookmarks(&self, bookmark: &str) {
        let mut bookmarks = self.get_bookmarks();
        if let Some(i) = bookmarks.iter().position(|x| x == bookmark) {
            bookmarks.remove(i);
            self.set_bookmarks(bookmarks);
        }
    }

    pub fn reorder_bookmarks(&self, a: usize, b: usize) {
        if a == b {
            return;
        }

        let mut bookmarks = self.get_bookmarks();
        let temp = bookmarks[a].clone();
        if a < b {
            for i in a + 1..=b {
                bookmarks[i - 1] = bookmarks[i].clone();
            }
        } else {
            for i in (b + 1..=a).rev() {
                bookmarks[i] = bookmarks[i - 1].clone();
            }
        }
        bookmarks[b] = temp;

        self.set_bookmarks(bookmarks);
    }
}
