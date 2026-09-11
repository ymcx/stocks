use crate::{
    api::{Interval, Range},
    custom_window::Window,
};
use adw::{Application, NavigationPage, NavigationSplitView, NavigationView, prelude::*};
use gtk::{
    Box, Button, DrawingArea, Orientation,
    cairo::Context,
    gio::Settings,
    glib::{self, ExitCode},
};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

mod api;
mod custom_window;

const APP_NAME: &str = "Stocks";
const APP_ID: &str = "com.ymcx.Stocks";

fn main() -> ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn draw_func(
    _area: &DrawingArea,
    cr: &Context,
    width: i32,
    height: i32,
    data: &(String, String, String, String, f64, Vec<f64>),
) {
    let (_symbol, _long_name, _short_name, _currency, _price, open) = data;

    if open.len() == 0 {
        return;
    }
    let chart_width = width;
    let chart_height = height;
    if chart_width <= 0 || chart_height <= 0 {
        return;
    }
    let max = open
        .iter()
        .max_by(|&&a, &b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min = open
        .iter()
        .min_by(|&&a, &b| a.partial_cmp(b).unwrap())
        .unwrap();

    let coords: Vec<(f64, f64)> = open
        .iter()
        .enumerate()
        .map(|(i, &price)| {
            let x = (i as f64) / ((open.len() - 1) as f64) * chart_width as f64;
            let y = chart_height as f64 - (price - min) / (max - min) * chart_height as f64;
            println!("{} {}", x, y);

            (x, y)
        })
        .collect();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_width(3.0);
    cr.move_to(0.0, 0.0);
    for (x, y) in coords {
        cr.line_to(x, y);
    }
    cr.stroke().unwrap();
}

fn create_page(data: (String, String, String, String, f64, Vec<f64>)) -> NavigationPage {
    let content = Box::new(Orientation::Vertical, 12);

    let chart = DrawingArea::builder().hexpand(true).vexpand(true).build();
    chart.set_draw_func(move |a, b, c, d| draw_func(a, b, c, d, &data));

    content.append(&chart);
    let page = NavigationPage::new(&content, "stock page");

    page
}

fn build_ui(app: &Application) {
    let window = Window::new(app);

    let split = NavigationSplitView::new();

    let boxi = Box::new(Orientation::Vertical, 12);
    let settings = Settings::new(APP_ID);

    let bookmarks: Vec<String> = settings
        .value("bookmarks")
        .get()
        .expect("bookmarks should be an array of strings");

    let navigation = NavigationView::new();
    let content_page = NavigationPage::new(&navigation, "content page");

    let sidebar = NavigationPage::new(&boxi, "sidebar");

    split.set_sidebar(Some(&sidebar));
    split.set_content(Some(&content_page));

    let (sender, receiver) = async_channel::bounded(1);

    for bookmark in bookmarks {
        let bookmark_clone = bookmark.clone();
        let button = Button::builder()
            .label(bookmark)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let sender = sender.clone();
        button.connect_clicked(move |_| {
            let bookmark_clone = bookmark_clone.clone();
            runtime().spawn(glib::clone!(
                #[strong]
                sender,
                async move {
                    let response =
                        api::fetch_stock(&bookmark_clone, Interval::OneDay, Range::OneMonth)
                            .await
                            .unwrap();
                    sender
                        .send(response)
                        .await
                        .expect("The channel needs to be open.");
                }
            ));
        });

        boxi.append(&button);
    }

    glib::spawn_future_local(async move {
        while let Ok(response) = receiver.recv().await {
            let page = create_page(response);
            navigation.push(&page);
        }
    });

    window.set_child(Some(&split));
    window.present();
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}
