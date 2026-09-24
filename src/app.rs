use crate::{
    APP_ID, APP_NAME, APP_VERSION,
    api::{self, Range, Stock},
    settings::Settings,
    window::Window,
};
use adw::{
    AboutDialog, Application, Bin, Breakpoint, BreakpointCondition, Dialog, HeaderBar,
    NavigationPage, NavigationSplitView, NavigationView, StatusPage, ToolbarView,
    gtk::{
        Box, Button, DrawingArea, Entry, Label, ListBox, ListBoxRow, MenuButton, Orientation,
        PolicyType, ScrolledWindow, SelectionMode,
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
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::OnceLock,
};
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

fn create_about_dialog() -> AboutDialog {
    let dial = adw::AboutDialog::builder()
        .application_icon(APP_ID)
        .application_name(APP_NAME)
        .developer_name("ymcx")
        .issue_url("https://github.com/ymcx/stocks/issues")
        .license_type(gtk::License::MitX11)
        .version(APP_VERSION)
        .website("https://github.com/ymcx/stocks/issues")
        .build();
    dial
}

fn create_row(sidebar: &ListBox, bookmark: &str, settings: &Settings) -> ListBoxRow {
    let row = ListBoxRow::builder().activatable(true).build();
    let row_box = Box::new(Orientation::Horizontal, 12);
    let label = Label::new(Some(bookmark));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.set_ellipsize(EllipsizeMode::End);

    row_box.append(&label);
    row.set_child(Some(&row_box));

    let menu = Menu::new();
    menu.append(Some("Delete"), Some("row.delete"));
    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover.set_parent(&row);
    let gesture = GestureClick::new();
    gesture.set_button(3);
    gesture.connect_pressed({
        let popover = popover.clone();
        move |_, _, x, y| {
            popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        }
    });

    row.add_controller(gesture);

    let actions = SimpleActionGroup::new();
    let delete_action = SimpleAction::new("delete", None);
    actions.add_action(&delete_action);
    let settings_value = settings.clone();
    let b = bookmark.to_string();
    let rr = row.clone();
    let ss = sidebar.clone();
    delete_action.connect_activate(move |_, _| {
        ss.remove(&rr);
        settings_value.del_bookmarks(&b);
    });
    row.insert_action_group("row", Some(&actions));

    row
}
fn create_sidebar(
    window: &Window,
    sidebar: ListBox,
    dialog: Dialog,
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

    let sidebar_bin = Bin::new();
    sidebar_bin.set_child(Some(&sidebar));

    let sidebar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&sidebar_bin)
        .build();
    sidebar_view.set_content(Some(&sidebar_scroll));

    sidebar.connect_row_activated({
        let sender = sender.clone();
        let settings = settings.clone();
        move |_, row| {
            let bookmarks = settings.get_bookmarks();
            if let Some(symbol) = bookmarks.get(row.index() as usize) {
                runtime().spawn(fetch_into(symbol.clone(), DEFAULT_RANGE, sender.clone()));
            }
        }
    });

    for bookmark in settings.get_bookmarks() {
        let row = create_row(&sidebar, &bookmark, &settings);
        sidebar.append(&row);
    }

    let sidebar_page = NavigationPage::new(&sidebar_view, "Bookmarks");
    sidebar_page
}
fn addf(symbol: &str, settings: &Settings, value_sidebar: &ListBox) {
    settings.add_bookmarks(symbol);
    let row = create_row(value_sidebar, symbol, settings);
    value_sidebar.append(&row);
}
fn create_dialog(settings: Settings, value_sidebar: ListBox) -> Dialog {
    let entry = Entry::builder()
        .placeholder_text("Stock symbol")
        .hexpand(true)
        .activates_default(true)
        .build();

    let error_label = Label::builder()
        .label("No stock found with that symbol")
        .xalign(0.0)
        .visible(false)
        .css_classes(["error"])
        .build();

    let content = Box::new(Orientation::Vertical, 6);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.append(&entry);
    content.append(&error_label);

    let header = HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();

    let cancel = Button::builder()
        .label("_Cancel")
        .use_underline(true)
        .valign(gtk::Align::Center)
        .build();
    let add_button = Button::builder()
        .label("_Add")
        .use_underline(true)
        .valign(gtk::Align::Center)
        .css_classes(["suggested-action"])
        .build();

    header.pack_start(&cancel);
    header.pack_end(&add_button);

    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&content));

    let dialog = Dialog::builder()
        .title("Add Bookmark")
        .content_width(420)
        .default_widget(&add_button)
        .child(&toolbar)
        .build();

    let generation = Rc::new(Cell::new(0_u64));

    cancel.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
        }
    });

    dialog.connect_closed({
        let generation = generation.clone();
        move |_| {
            generation.set(generation.get() + 1);
        }
    });

    entry.connect_changed({
        let entry = entry.clone();
        let error_label = error_label.clone();
        move |_| {
            entry.remove_css_class("error");
            error_label.set_visible(false);
        }
    });

    add_button.connect_clicked({
        let dialog = dialog.clone();
        let settings = settings.clone();
        let sidebar = value_sidebar.clone();
        let entry = entry.clone();
        let error_label = error_label.clone();
        let add_button = add_button.clone();
        let generation = generation.clone();
        move |_| {
            let symbol = entry.text().trim().to_uppercase();

            if symbol.is_empty() {
                entry.add_css_class("error");
                error_label.set_visible(true);
                entry.grab_focus();
                return;
            }

            entry.set_sensitive(false);
            add_button.set_sensitive(false);

            let (sender, receiver) = async_channel::bounded(1);
            let check = symbol.clone();
            runtime().spawn(async move {
                let _ = sender.send(api::valid(&check).await).await;
            });

            glib::spawn_future_local({
                let dialog = dialog.clone();
                let settings = settings.clone();
                let sidebar = sidebar.clone();
                let entry = entry.clone();
                let error_label = error_label.clone();
                let add_button = add_button.clone();
                let generation = generation.clone();
                let current = generation.get();
                async move {
                    let valid = receiver.recv().await.unwrap_or(false);

                    entry.set_sensitive(true);
                    add_button.set_sensitive(true);

                    if generation.get() != current {
                        return;
                    }

                    if valid {
                        addf(&symbol, &settings, &sidebar);
                        dialog.close();
                    } else {
                        entry.add_css_class("error");
                        error_label.set_visible(true);
                        entry.grab_focus();
                    }
                }
            });
        }
    });

    dialog.connect_map({
        let entry = entry.clone();
        let error_label = error_label.clone();
        let add_button = add_button.clone();
        let generation = generation.clone();
        move |_| {
            generation.set(generation.get() + 1);
            entry.set_sensitive(true);
            add_button.set_sensitive(true);
            entry.remove_css_class("error");
            error_label.set_visible(false);
            entry.grab_focus();
        }
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

    let side = ListBox::new();
    side.add_css_class("navigation-sidebar");
    side.set_selection_mode(SelectionMode::Single);
    side.set_activate_on_single_click(true);

    let navigation = NavigationView::new();

    let dialog = create_dialog(settings.clone(), side.clone());
    let sidebar_page = create_sidebar(&window, side.clone(), dialog, settings, sender.clone(), app);
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

    glib::idle_add_local_once({
        let side = side.clone();
        move || side.unselect_all()
    });
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
