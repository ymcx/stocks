use gtk::glib::ExitCode;

mod api;
mod app;
mod window;

const APP_NAME: &str = "Stocks";
const APP_ID: &str = "com.ymcx.Stocks";

fn main() -> ExitCode {
    app::run()
}
