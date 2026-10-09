use adw::gtk::{gio, glib::ExitCode};
use std::env;

mod api;
mod app;
mod color;
mod settings;

const APP_VERSION: &str = "0.1.0";
const APP_NAME: &str = "Stocks";
const APP_ID: &str = "com.ymcx.Stocks";

fn main() -> ExitCode {
    if env::var_os("GSETTINGS_SCHEMA_DIR").is_none() {
        unsafe {
            env::set_var("GSETTINGS_SCHEMA_DIR", concat!(env!("OUT_DIR"), "/schemas"));
        }
    }

    gio::resources_register_include!("compiled.gresource").unwrap();
    app::run()
}
