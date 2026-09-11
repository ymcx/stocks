use crate::APP_NAME;
use adw::subclass::prelude::*;
use gtk::{
    gio::Settings,
    glib::{self, Propagation},
};
use std::cell::OnceCell;

#[derive(Default)]
pub struct Window {
    pub settings: OnceCell<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = APP_NAME;
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.setup_settings();
        obj.load_window_size();
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {
    fn close_request(&self) -> Propagation {
        self.obj()
            .save_window_size()
            .expect("Failed to save window state");
        Propagation::Proceed
    }
}

impl ApplicationWindowImpl for Window {}
