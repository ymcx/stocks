use crate::APP_ID;
use adw::gtk::gio::{self, prelude::SettingsExtManual};

#[derive(Clone)]
pub struct Settings {
    settings: gio::Settings,
}

impl Settings {
    pub fn new() -> Self {
        let settings = gio::Settings::new(APP_ID);
        Self { settings }
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
}
