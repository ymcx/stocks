use crate::{
    APP_ID, APP_NAME,
    api::{self, Range, Stock},
    settings::Settings,
    window::Window,
};
use adw::{
    AlertDialog, Application, Breakpoint, BreakpointCondition, Dialog, HeaderBar, NavigationPage,
    NavigationSplitView, NavigationView, ResponseAppearance, StatusPage, ToolbarView,
    gtk::{
        Box, Button, DrawingArea, Entry, Label, MenuButton, Orientation, PolicyType,
        ScrolledWindow,
        cairo::Context,
        gio::Menu,
        glib::{self, ExitCode},
        pango::EllipsizeMode,
    },
    prelude::*,
};
use async_channel::Sender;
use gtk::{
    GestureClick, PopoverMenu,
    gdk::Rectangle,
    gio::{SimpleAction, SimpleActionGroup},
};
use std::{cell::RefCell, rc::Rc, sync::OnceLock};
use tokio::runtime::Runtime;

const DEFAULT_RANGE: Range = Range::OneMonth;

pub fn run() -> ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("setting up the tokio runtime should succeed"))
}

async fn fetch_into(symbol: String, range: Range, sender: Sender<Stock>) {
    match api::fetch_stock(&symbol, range).await {
        Ok(stock) => {
            let _ = sender.send(stock).await;
        }
        Err(error) => eprintln!("Failed to fetch {symbol}: {error}"),
    }
}
fn create_navigation_empty(navigation: NavigationView) -> NavigationPage {
    let placeholder = StatusPage::builder()
        .title(APP_NAME)
        .description("Select a stock from the sidebar")
        .icon_name("view-list-symbolic")
        .build();

    let placeholder_view = ToolbarView::new();
    placeholder_view.add_top_bar(&HeaderBar::new());
    placeholder_view.set_content(Some(&placeholder));
    navigation.add(&NavigationPage::new(&placeholder_view, APP_NAME));

    let content_page = NavigationPage::new(&navigation, APP_NAME);
    content_page
}

fn create_about_dialog() -> Dialog {
    let boxi = Box::new(Orientation::Vertical, 12);
    boxi.set_margin_top(24);
    boxi.set_margin_bottom(24);
    boxi.set_margin_start(24);
    boxi.set_margin_end(24);
    let title = Label::new(Some("lol"));
    boxi.append(&title);
    let header = HeaderBar::new();
    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&boxi));
    let dialog = Dialog::new();
    dialog.set_child(Some(&toolbar));

    dialog
}
fn create_button(
    sidebar: &Box,
    bookmark: &str,
    settings: &Settings,
    sender: &Sender<Stock>,
) -> Button {
    let button = Button::builder()
        .label(bookmark)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let menu = Menu::new();
    menu.append(Some("Delete"), Some("button.delete"));
    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover.set_parent(&button);
    let gesture = GestureClick::new();
    gesture.set_button(3);
    gesture.connect_pressed({
        let popover = popover.clone();
        move |_, _, x, y| {
            popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        }
    });

    button.add_controller(gesture);

    let actions = SimpleActionGroup::new();
    let delete_action = SimpleAction::new("delete", None);
    actions.add_action(&delete_action);
    let settings_value = settings.clone();
    let b = bookmark.to_string();
    let bb = button.clone();
    let ss = sidebar.clone();
    delete_action.connect_activate(move |_, _| {
        ss.remove(&bb);
        settings_value.del_bookmarks(&b);
    });
    button.insert_action_group("button", Some(&actions));

    let b = bookmark.to_string();
    button.connect_clicked({
        let sender = sender.clone();
        move |_| {
            let symbol = b.clone();
            let sender = sender.clone();
            runtime().spawn(fetch_into(symbol.to_string(), DEFAULT_RANGE, sender));
        }
    });
    button
}
fn create_sidebar(
    window: &Window,
    sidebar: Box,
    dialog: AlertDialog,
    settings: Settings,
    sender: Sender<Stock>,
    app: &Application,
) -> NavigationPage {
    let sidebar_view = ToolbarView::new();
    let header = &HeaderBar::new();
    let menu = Menu::new();
    menu.append(Some("About"), Some("app.about"));

    let new_action = SimpleAction::new("about", None);
    let value = window.clone();
    new_action.connect_activate(move |_, _| {
        let di = create_about_dialog();
        di.present(Some(&value));
    });
    app.add_action(&new_action);

    let add_button = Button::builder().icon_name("list-add-symbolic").build();

    let value = window.clone();
    add_button.connect_clicked(move |_| {
        dialog.present(Some(&value));
    });

    let menu_button = MenuButton::builder()
        .label("Menu")
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();
    header.pack_start(&add_button);
    header.pack_end(&menu_button);

    sidebar_view.add_top_bar(header);
    sidebar_view.set_content(Some(&sidebar));

    let sidebar_page = NavigationPage::new(&sidebar_view, "Bookmarks");
    let bookmarks: Vec<String> = settings.get_bookmarks();
    // let sender_value=sender.clone();
    for bookmark in bookmarks {
        let button = create_button(&sidebar, &bookmark, &settings, &sender);

        sidebar.append(&button);
    }

    sidebar_page
}
fn add(entry: &Entry, settings: &Settings, value_sender: &Sender<Stock>, value_sidebar: &Box) {
    let bookmark = entry.text().to_uppercase();
    settings.add_bookmarks(&bookmark);
    let button = create_button(value_sidebar, &bookmark, settings, value_sender);
    value_sidebar.append(&button);
}
fn create_dialog(
    settings: Settings,
    value_sender: Sender<Stock>,
    value_sidebar: Box,
) -> AlertDialog {
    let entry = Entry::builder()
        .placeholder_text("lol")
        .hexpand(true)
        .build();

    let dialog = AlertDialog::new(Some("header"), Some("body"));
    dialog.set_extra_child(Some(&entry));
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("add", "Add");
    dialog.set_default_response(Some("add"));
    dialog.set_response_appearance("add", ResponseAppearance::Suggested);
    dialog.set_close_response("cancel");

    let value_dialog = dialog.clone();
    let value_value_sidebar = value_sidebar.clone();
    let value_value_sender = value_sender.clone();
    let value_settings = settings.clone();
    let value_entry = entry.clone();
    entry.connect_activate({
        move |_| {
            add(
                &value_entry,
                &value_settings,
                &value_value_sender,
                &value_value_sidebar,
            );
            value_dialog.close();
        }
    });
    let value_dialog = dialog.clone();
    let value_value_sidebar = value_sidebar.clone();
    let value_value_sender = value_sender.clone();
    let value_settings = settings.clone();
    let value_entry = entry.clone();
    dialog.connect_response(None, move |_dia, res| {
        if res == "add" {
            add(
                &value_entry,
                &value_settings,
                &value_value_sender,
                &value_value_sidebar,
            );
        } else {
            value_dialog.close();
        }
    });
    let value_entry = entry.clone();
    dialog.connect_map(move |_| {
        value_entry.grab_focus();
    });

    dialog
}

fn build_ui(app: &Application) {
    let window = Window::new(app);
    window.set_title(Some(APP_NAME));

    let split = NavigationSplitView::new();

    let breakpoint = Breakpoint::new(
        BreakpointCondition::parse("max-width: 550sp").expect("valid breakpoint condition"),
    );
    breakpoint.add_setter(&split, "collapsed", Some(&true.to_value()));
    window.add_breakpoint(breakpoint);

    let settings = Settings::new();

    let (sender, receiver) = async_channel::bounded(1);

    let side = Box::new(Orientation::Vertical, 12);
    let navigation = NavigationView::new();

    let dialog = create_dialog(settings.clone(), sender.clone(), side.clone());
    let sidebar_page = create_sidebar(&window, side, dialog, settings, sender.clone(), app);
    let navigation_page = create_navigation_empty(navigation.clone());

    split.set_sidebar(Some(&sidebar_page));
    split.set_content(Some(&navigation_page));

    glib::spawn_future_local({
        let navigation = navigation.clone();
        let split = split.clone();
        async move {
            while let Ok(stock) = receiver.recv().await {
                navigation.replace(&[create_stock_page(stock)]);
                split.set_show_content(true);
            }
        }
    });

    window.set_content(Some(&split));
    window.present();
}

fn create_stock_page(stock: Stock) -> NavigationPage {
    let state = Rc::new(RefCell::new(stock));

    let header = Box::new(Orientation::Horizontal, 12);
    header.set_margin_top(12);
    header.set_margin_bottom(6);
    header.set_margin_start(12);
    header.set_margin_end(12);

    let title = Label::new(None);
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.set_ellipsize(EllipsizeMode::End);
    title.add_css_class("title-3");

    let price = Label::new(None);
    price.set_xalign(1.0);
    price.add_css_class("title-2");

    header.append(&title);
    header.append(&price);

    let chart = DrawingArea::builder().hexpand(true).vexpand(true).build();
    chart.set_draw_func({
        let state = state.clone();
        move |area, cr, width, height| {
            draw_chart(area, cr, width, height, &state.borrow().open);
        }
    });

    let ranges = Box::new(Orientation::Horizontal, 6);
    ranges.set_homogeneous(true);
    ranges.set_margin_top(6);
    ranges.set_margin_bottom(12);
    ranges.set_margin_start(12);
    ranges.set_margin_end(12);

    let (sender, receiver) = async_channel::bounded(1);

    for range in Range::ALL {
        let button = Button::builder().label(range.as_str()).build();

        button.connect_clicked({
            let state = state.clone();
            let sender = sender.clone();
            move |_| {
                let symbol = state.borrow().symbol.clone();
                let sender = sender.clone();
                runtime().spawn(fetch_into(symbol, range, sender));
            }
        });

        ranges.append(&button);
    }

    glib::spawn_future_local({
        let state = state.clone();
        let chart = chart.clone();
        let title = title.clone();
        let price = price.clone();
        async move {
            while let Ok(stock) = receiver.recv().await {
                title.set_label(stock.name());
                price.set_label(&format_price(stock.price, &stock.currency));
                *state.borrow_mut() = stock;
                chart.queue_draw();
            }
        }
    });

    let content = Box::new(Orientation::Vertical, 0);

    let ranges_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::External)
        .vscrollbar_policy(PolicyType::Never)
        .propagate_natural_height(true)
        .child(&ranges)
        .build();

    content.append(&header);
    content.append(&chart);
    content.append(&ranges_scroll);

    let stock = state.borrow();
    title.set_label(stock.name());
    price.set_label(&format_price(stock.price, &stock.currency));

    let view = ToolbarView::new();
    view.add_top_bar(&HeaderBar::new());
    view.set_content(Some(&content));

    NavigationPage::new(&view, &stock.symbol)
}

fn draw_chart(area: &DrawingArea, cr: &Context, width: i32, height: i32, open: &[f64]) {
    if open.is_empty() || width <= 0 || height <= 0 {
        return;
    }

    let width = f64::from(width);
    let height = f64::from(height);
    let padding = 8.0;
    let chart_width = width - 2.0 * padding;
    let chart_height = height - 2.0 * padding;

    if chart_width <= 0.0 || chart_height <= 0.0 {
        return;
    }

    let min = open.iter().copied().fold(f64::INFINITY, f64::min);
    let max = open.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;

    let step = if open.len() > 1 {
        chart_width / (open.len() - 1) as f64
    } else {
        0.0
    };

    let coords: Vec<(f64, f64)> = open
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let x = padding + index as f64 * step;
            let y = if span > 0.0 {
                padding + (1.0 - (value - min) / span) * chart_height
            } else {
                padding + chart_height / 2.0
            };
            (x, y)
        })
        .collect();

    let first = coords[0];
    let last = coords[coords.len() - 1];
    let baseline = height - padding;

    let color = area.color();
    let (red, green, blue) = (
        f64::from(color.red()),
        f64::from(color.green()),
        f64::from(color.blue()),
    );

    cr.move_to(first.0, baseline);
    for &(x, y) in &coords {
        cr.line_to(x, y);
    }
    cr.line_to(last.0, baseline);
    cr.close_path();
    cr.set_source_rgba(red, green, blue, 0.15);
    cr.fill().expect("filling the chart path should succeed");

    cr.move_to(first.0, first.1);
    for &(x, y) in &coords[1..] {
        cr.line_to(x, y);
    }
    cr.set_source_rgba(red, green, blue, 1.0);
    cr.set_line_width(2.0);
    cr.stroke().expect("stroking the chart path should succeed");
}

fn format_price(price: f64, currency: &str) -> String {
    let symbol = match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "KRW" => "₩",
        "INR" => "₹",
        _ => "",
    };

    if symbol.is_empty() {
        format!("{price:.2} {currency}").trim_end().to_owned()
    } else {
        format!("{symbol}{price:.2}")
    }
}
