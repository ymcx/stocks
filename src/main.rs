use adw::gtk::{gio, glib::ExitCode};

mod api;
mod app;
mod settings;

const APP_VERSION: &str = "0.1.0";
const APP_NAME: &str = "Stocks";
const APP_ID: &str = "com.ymcx.Stocks";

fn main() -> ExitCode {
    gio::resources_register_include!("compiled.gresource").unwrap();
    app::run()
}
