use crate::APP_ID;
use adw::gtk::gio::{Settings, prelude::SettingsExtManual};

pub struct XDSettings {
    settings: Settings,
}

impl XDSettings {
    pub fn new() -> Self {
        let settings = Settings::new(APP_ID);
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
}
