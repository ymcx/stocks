use adw::gtk::glib::ExitCode;

mod api;
mod app;
mod settings;
mod window;

const APP_NAME: &str = "Stocks";
const APP_ID: &str = "com.ymcx.Stocks";

fn main() -> ExitCode {
    app::run()
}
