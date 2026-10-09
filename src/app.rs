use crate::{
    APP_ID, APP_NAME, APP_VERSION,
    api::{Range, Stock},
    color::ColorScheme,
    settings::Settings,
};
use adw::{
    AboutDialog, Application, ApplicationWindow, Bin, Breakpoint, BreakpointCondition,
    BreakpointConditionLengthType, Dialog, EntryRow, HeaderBar, LengthUnit, NavigationPage,
    NavigationSplitView, NavigationView, PreferencesGroup, PreferencesPage, StatusPage,
    ToolbarView,
    gtk::{
        Box, Button, DragSource, DrawingArea, DropTarget, GestureClick, Label, License, ListBox,
        ListBoxRow, MenuButton, Orientation, PolicyType, PopoverMenu, ScrolledWindow,
        WidgetPaintable,
        cairo::{Context, FontSlant, FontWeight, LinearGradient},
        gdk::{ContentProvider, DragAction, Rectangle},
        gio::{Menu, SimpleAction, SimpleActionGroup},
        glib::{self, ExitCode, Propagation},
        pango::EllipsizeMode,
    },
    prelude::*,
};
use gtk::EventControllerMotion;
use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::OnceLock};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

pub fn run() -> ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn create_placeholder_page() -> NavigationPage {
    let header_bar = HeaderBar::new();
    let status_page = StatusPage::builder()
        .title(APP_NAME)
        .description("Select a stock from the sidebar")
        .icon_name("view-list-symbolic")
        .build();

    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);
    toolbar_view.set_content(Some(&status_page));

    let navigation_page = NavigationPage::builder()
        .title(APP_NAME)
        .tag("home")
        .child(&toolbar_view)
        .build();

    navigation_page
}

fn create_about_dialog() -> AboutDialog {
    let about_dialog = AboutDialog::builder()
        .application_icon(APP_ID)
        .application_name(APP_NAME)
        .developer_name("ymcx")
        .issue_url("https://github.com/ymcx/stocks/issues")
        .license_type(License::MitX11)
        .version(APP_VERSION)
        .website("https://github.com/ymcx/stocks")
        .build();

    about_dialog
}

fn create_sidebar_button(sidebar: &ListBox, bookmark: &str, settings: &Settings) -> ListBoxRow {
    let list_box_row = ListBoxRow::new();

    let row_box = Box::builder()
        .margin_bottom(8)
        .margin_end(8)
        .margin_start(8)
        .margin_top(8)
        .build();
    list_box_row.set_child(Some(&row_box));

    let label = Label::builder()
        .ellipsize(EllipsizeMode::End)
        .label(bookmark)
        .build();
    row_box.append(&label);

    // TODO: Automatically select Delete by default
    let menu = Menu::new();
    menu.append(Some("Delete"), Some("row.delete"));

    let popover_menu = PopoverMenu::builder()
        .menu_model(&menu)
        .has_arrow(false)
        .build();
    popover_menu.set_parent(&list_box_row);

    let gesture_click = GestureClick::builder().button(3).build();
    gesture_click.connect_pressed(move |_, _, x, y| {
        let rectangle = Rectangle::new(x as i32, y as i32, 120, 0);
        popover_menu.set_pointing_to(Some(&rectangle));
        popover_menu.popup();
    });
    list_box_row.add_controller(gesture_click);

    let simple_action_group = SimpleActionGroup::new();
    list_box_row.insert_action_group("row", Some(&simple_action_group));

    let simple_action = SimpleAction::new("delete", None);
    simple_action.connect_activate({
        let list_box_row = list_box_row.clone();
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        move |_, _| {
            let index = list_box_row.index() as usize;
            let is_current_or_home = sidebar.selected_row().is_none_or(|i| i == list_box_row);
            if is_current_or_home {
                glib::idle_add_local_once({
                    let sidebar = sidebar.clone();
                    move || {
                        sidebar.unselect_all();
                    }
                });
            }

            sidebar.remove(&list_box_row);
            settings.remove_bookmark(index);
        }
    });
    simple_action_group.add_action(&simple_action);

    let drag_source = DragSource::builder().actions(DragAction::MOVE).build();
    drag_source.connect_prepare({
        let value = list_box_row.to_value();
        move |_, _, _| {
            let content_provider = ContentProvider::for_value(&value);
            Some(content_provider)
        }
    });
    drag_source.connect_drag_begin(move |drag_source, _| {
        let widget_paintable = WidgetPaintable::new(Some(&row_box));
        drag_source.set_icon(Some(&widget_paintable), -15, -15);
    });
    list_box_row.add_controller(drag_source);

    let drop_target = DropTarget::new(ListBoxRow::static_type(), DragAction::MOVE);
    drop_target.connect_drop({
        let sidebar = sidebar.clone();
        let settings = settings.clone();
        move |target, source, _, _| {
            let Ok(source_row) = source.get::<ListBoxRow>() else {
                return false;
            };
            let Some(target_row) = target
                .widget()
                .and_then(|i| i.downcast::<ListBoxRow>().ok())
            else {
                return false;
            };

            let source_index = source_row.index() as usize;
            let target_index = target_row.index() as usize;
            if source_index == target_index {
                return true;
            }

            sidebar.remove(&source_row);
            sidebar.insert(&source_row, target_index as i32);
            settings.reorder_bookmarks(source_index, target_index);

            true
        }
    });
    list_box_row.add_controller(drop_target);

    list_box_row
}

fn create_sidebar_page(
    sidebar: &ListBox,
    settings: &Settings,
    sender: &Sender<Stock>,
    range_default: &Rc<RefCell<Range>>,
) -> NavigationPage {
    let toolbar_view = ToolbarView::new();

    let header_bar = HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let bin = Bin::builder().child(sidebar).build();

    let scrolled_window = ScrolledWindow::builder().child(&bin).build();
    toolbar_view.set_content(Some(&scrolled_window));

    let menu = Menu::new();
    menu.append(Some("About"), Some("app.about"));

    let add_button = Button::builder()
        .icon_name("bookmark-new-symbolic")
        .action_name("app.add")
        .build();
    header_bar.pack_start(&add_button);

    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();
    header_bar.pack_end(&menu_button);

    sidebar.connect_row_activated({
        let sender = sender.clone();
        let settings = settings.clone();
        let range_default = range_default.clone();
        move |_, list_box_row| {
            let bookmarks = settings.get_bookmarks();
            let index = list_box_row.index() as usize;
            if let Some(bookmark) = bookmarks.get(index) {
                let bookmark = bookmark.clone();
                let sender = sender.clone();
                let range_default = *range_default.borrow();
                let runtime = runtime();
                runtime.spawn(Stock::fetch_and_send(bookmark, range_default, sender));
            }
        }
    });

    for bookmark in settings.get_bookmarks() {
        let button = create_sidebar_button(sidebar, &bookmark, settings);
        sidebar.append(&button);
    }

    let navigation_page = NavigationPage::new(&toolbar_view, APP_NAME);

    navigation_page
}

fn create_add_dialog(settings: &Settings, sidebar: &ListBox) -> Dialog {
    let preferences_page = PreferencesPage::new();

    let preferences_group = PreferencesGroup::new();
    preferences_page.add(&preferences_group);

    // TODO: empty text on dialog open
    let entry_row = EntryRow::builder()
        .title("Stock Symbol")
        .activates_default(true)
        .build();
    entry_row.connect_changed({
        let entry = entry_row.clone();
        move |_| {
            entry.remove_css_class("error");
        }
    });
    preferences_group.add(&entry_row);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(90)
        .margin_end(0)
        .margin_start(0)
        .margin_top(0)
        .build();
    content.append(&preferences_page);

    let header = HeaderBar::builder()
        .show_end_title_buttons(false)
        .show_start_title_buttons(false)
        .build();

    let toolbar = ToolbarView::builder().content(&content).build();
    toolbar.add_top_bar(&header);

    let dialog = Dialog::builder()
        .title("Add Bookmark")
        .content_width(640)
        .child(&toolbar)
        .build();

    // TODO: disable add button if entry is empty
    let button_add = Button::builder()
        .label("_Add")
        .use_underline(true)
        .css_classes(["suggested-action"])
        .build();
    button_add.connect_clicked({
        let dialog = dialog.clone();
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        let entry_row = entry_row.clone();
        move |_| {
            let bookmark = entry_row.text().trim().to_uppercase();
            if bookmark.is_empty() {
                entry_row.add_css_class("error");
                entry_row.grab_focus();
                return;
            }

            let (sender, mut receiver) = mpsc::channel(1);
            let range = Range::OneMonth;
            let runtime = runtime();
            runtime.spawn(Stock::fetch_and_send(bookmark.clone(), range, sender));

            glib::spawn_future_local({
                let dialog = dialog.clone();
                let settings = settings.clone();
                let sidebar = sidebar.clone();
                let entry_row = entry_row.clone();
                async move {
                    let stock = receiver.recv().await;
                    if stock.is_none() {
                        entry_row.add_css_class("error");
                        entry_row.grab_focus();
                        return;
                    }

                    let button = create_sidebar_button(&sidebar, &bookmark, &settings);
                    sidebar.append(&button);
                    settings.add_bookmark(&bookmark);
                    dialog.close();
                }
            });
        }
    });
    header.pack_end(&button_add);
    dialog.set_default_widget(Some(&button_add));

    let button_cancel = Button::builder()
        .label("_Cancel")
        .use_underline(true)
        .build();
    button_cancel.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
        }
    });
    header.pack_start(&button_cancel);

    dialog.connect_map(move |_| {
        entry_row.set_text("");
        entry_row.grab_focus();
    });

    dialog
}

fn create_stock_page_header_title(stock: &Stock, color_scheme: &ColorScheme) -> Box {
    let container = Box::builder().build();

    let color = color_scheme.foreground.as_attrs(true, Some(24));
    let title = stock.short_name.to_string();
    let title = Label::builder().attributes(&color).label(title).build();
    container.append(&title);

    container
}

fn create_stock_page_header_price(stock: &Stock, color_scheme: &ColorScheme) -> Box {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let color = color_scheme.foreground.as_attrs(true, Some(16));
    let price = format!("{} {}", stock.regular_market_price, stock.currency);
    let price = Label::builder().attributes(&color).label(price).build();
    container.append(&price);

    let color = color_scheme
        .change(stock.fullday_change)
        .as_attrs(true, Some(16));
    let change = format!(
        "{:.2}% ({})",
        stock.fullday_change_percent, stock.fullday_change
    );
    let change = Label::builder().attributes(&color).label(change).build();
    container.append(&change);

    container
}

fn create_stock_page_header_metadata_item(
    label: &str,
    value: f64,
    color_scheme: &ColorScheme,
) -> Box {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();

    let color = color_scheme.foreground_dim.as_attrs(false, Some(12));
    let label = Label::builder().attributes(&color).label(label).build();
    container.append(&label);

    let color = color_scheme.foreground.as_attrs(true, Some(12));
    let value = value.to_string();
    let value = Label::builder().attributes(&color).label(value).build();
    container.append(&value);

    container
}

fn create_stock_page_header_metadata(stock: &Stock, color_scheme: &ColorScheme) -> Box {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let high = stock.regular_market_day_high;
    let high = create_stock_page_header_metadata_item("High", high, color_scheme);
    container.append(&high);

    let low = stock.regular_market_day_low;
    let low = create_stock_page_header_metadata_item("Low", low, color_scheme);
    container.append(&low);

    let volume = stock.regular_market_volume as f64;
    let volume = create_stock_page_header_metadata_item("Volume", volume, color_scheme);
    container.append(&volume);

    container
}

fn create_stock_page_header(stock: &Stock, color_scheme: &ColorScheme) -> Box {
    let header = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let title = create_stock_page_header_title(stock, color_scheme);
    header.append(&title);

    let price = create_stock_page_header_price(stock, color_scheme);
    header.append(&price);

    let metadata = create_stock_page_header_metadata(stock, &color_scheme);
    header.append(&metadata);

    header
}

fn create_stock_page_chart(state: &Rc<RefCell<Stock>>) -> DrawingArea {
    let chart = DrawingArea::builder().vexpand(true).hexpand(true).build();
    let cursor = Rc::new(RefCell::new(None));
    let motion = EventControllerMotion::new();
    chart.add_controller(motion.clone());

    let chart_value = chart.clone();
    let cursor_value = cursor.clone();
    motion.connect_motion(move |_, x, y| {
        *cursor_value.borrow_mut() = Some((x, y));
        chart_value.queue_draw();
    });

    let chart_value = chart.clone();
    let cursor_value = cursor.clone();
    motion.connect_leave(move |_| {
        *cursor_value.borrow_mut() = None;
        chart_value.queue_draw();
    });

    let state_value = state.clone();
    chart.set_draw_func(move |_, cr, width, height| {
        let quote_close = &state_value.borrow().quote_close;
        draw_chart(cr, width, height, quote_close, &cursor);
    });

    chart
}

fn create_stock_page_ranges(
    stock: &Stock,
    sender: &Sender<Stock>,
    range_default: &Rc<RefCell<Range>>,
) -> ScrolledWindow {
    let ranges = Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .spacing(8)
        .build();

    for range in [
        Range::OneDay,
        Range::FiveDays,
        Range::OneMonth,
        Range::SixMonths,
        Range::Ytd,
        Range::OneYear,
        Range::FiveYears,
        Range::Max,
    ] {
        let symbol = stock.symbol.clone();
        let sender = sender.clone();
        let button = Button::builder().label(range.as_str()).build();
        ranges.append(&button);

        let range_default = range_default.clone();
        button.connect_clicked(move |_| {
            *range_default.borrow_mut() = range;
            let symbol = symbol.clone();
            let sender = sender.clone();
            let runtime = runtime();
            runtime.spawn(Stock::fetch_and_send(symbol, range, sender));
        });
    }

    let ranges_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::External)
        .vscrollbar_policy(PolicyType::Never)
        .margin_start(16)
        .margin_end(16)
        .child(&ranges)
        .build();

    ranges_window
}

fn create_stock_page(stock: Stock, range_default: &Rc<RefCell<Range>>) -> NavigationPage {
    let (sender, mut receiver) = mpsc::channel(1);
    let state = Rc::new(RefCell::new(stock.clone()));
    let color_scheme = ColorScheme::new();

    let navigation_page = NavigationPage::builder().title(&stock.symbol).build();

    let toolbar_view = ToolbarView::new();
    navigation_page.set_child(Some(&toolbar_view));

    let header_bar = HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(32)
        .margin_top(32)
        .margin_bottom(32)
        .build();
    toolbar_view.set_content(Some(&content));

    let header = create_stock_page_header(&stock, &color_scheme);
    content.append(&header);

    let chart = create_stock_page_chart(&state);
    content.append(&chart);

    let ranges = create_stock_page_ranges(&stock, &sender, range_default);
    content.append(&ranges);

    glib::spawn_future_local(async move {
        while let Some(stock) = receiver.recv().await {
            *state.borrow_mut() = stock;
            chart.queue_draw();
        }
    });

    navigation_page
}

fn get_quote_levels_step(estimate: f64) -> f64 {
    let magnitude = 10f64.powf(estimate.log10().floor());
    let normalized = estimate / magnitude;
    let step = if normalized < 1.5 {
        1.0
    } else if normalized < 3.0 {
        2.0
    } else if normalized < 7.0 {
        5.0
    } else {
        10.0
    };

    step * magnitude
}

fn get_decimals_for_step(step: f64) -> usize {
    let mut i = step;
    let mut decimals = 0;
    while i != i.round() {
        i *= 10.0;
        decimals += 1;
    }

    decimals
}

fn get_quote_levels(min: f64, max: f64) -> (Vec<f64>, usize) {
    let step_estimate = (max - min) / 6.0;
    let step = get_quote_levels_step(step_estimate);

    let mut levels = Vec::new();
    let mut i = (min / step).ceil() * step;
    while i <= max {
        levels.push(i);
        i += step;
    }

    let decimals = get_decimals_for_step(step);

    (levels, decimals)
}

fn draw_chart(
    cr: &Context,
    width: i32,
    height: i32,
    quote: &Vec<f64>,
    cursor: &Rc<RefCell<Option<(f64, f64)>>>,
) {
    if quote.len() == 0 || width == 0 || height == 0 {
        return;
    }

    let width = width as f64;
    let height = height as f64;
    let step = width / usize::max(quote.len() - 1, 1) as f64;

    let min = quote.iter().copied().fold(f64::INFINITY, f64::min);
    let max = quote.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;

    let coordinates: Vec<(f64, f64)> = quote
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = step * i as f64;
            let y = height - (value - min) / span * height;
            (x, y)
        })
        .collect();
    let first = coordinates[0];
    let last = coordinates[coordinates.len() - 1];

    let color_scheme = ColorScheme::new();
    let foreground = color_scheme.foreground.as_f64();
    let foreground_dim = color_scheme.foreground_dim.as_f64();
    let blue = color_scheme.blue.as_f64();

    cr.move_to(first.0, height);
    for &(x, y) in &coordinates {
        cr.line_to(x, y);
    }
    cr.line_to(last.0, height);
    cr.close_path();
    let gradient = LinearGradient::new(0.0, 0.0, 0.0, height);
    gradient.add_color_stop_rgba(0.0, blue.0, blue.1, blue.2, 0.1);
    gradient.add_color_stop_rgba(1.0, blue.0, blue.1, blue.2, 0.0);
    cr.set_source(gradient).unwrap();
    cr.fill().unwrap();

    cr.move_to(first.0, first.1);
    for &(x, y) in &coordinates[1..] {
        cr.line_to(x, y);
    }
    cr.set_source_rgba(blue.0, blue.1, blue.2, 1.0);
    cr.set_line_width(2.0);
    cr.stroke().unwrap();

    let (levels, decimals) = get_quote_levels(min, max);
    let text = format!("{:.decimals$}", levels[levels.len() - 1]);
    let extents = cr.text_extents(&text).unwrap();
    for level in levels {
        let x = width - extents.width() - 20.0;
        let y = height - (level - min) / span * height;
        let text = format!("{:.decimals$}", level);
        cr.set_source_rgba(foreground_dim.0, foreground_dim.1, foreground_dim.2, 1.0);
        cr.select_font_face("Adwaita Sans", FontSlant::Normal, FontWeight::Normal);
        cr.set_font_size(16.0);
        cr.move_to(x, y);
        cr.show_text(&text).unwrap();
    }

    let Some((x, y)) = cursor.borrow().and_then(|(x, _)| {
        let y = coordinates.iter().find(|&&(i, _)| x < i).map(|&(_, j)| j)?;
        Some((x, y))
    }) else {
        return;
    };

    cr.set_source_rgba(foreground_dim.0, foreground_dim.1, foreground_dim.2, 1.0);
    cr.set_line_width(1.0);
    cr.set_dash(&[4.0, 4.0], 0.0);
    cr.move_to(x, 0.0);
    cr.line_to(x, height);
    cr.move_to(0.0, y);
    cr.line_to(width, y);
    cr.stroke().unwrap();
    cr.set_dash(&[], 0.0);

    cr.set_source_rgba(foreground.0, foreground.1, foreground.2, 1.0);
    cr.arc(x, y, 5.0, 0.0, 2.0 * PI);
    cr.fill().unwrap();
}

fn build_ui(application: &Application) {
    let settings = Settings::new();
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title(APP_NAME)
        .default_width(settings.get_window_width())
        .default_height(settings.get_window_height())
        .maximized(settings.get_window_maximized())
        .build();

    let last_stock = settings.get_last_stock();
    let last_range = Range::parse(&settings.get_last_range()).unwrap_or_default();
    let symbol_default = Rc::new(RefCell::new(last_stock.clone()));
    let range_default = Rc::new(RefCell::new(last_range));

    application_window.connect_close_request({
        let settings = settings.clone();
        let symbol_default = symbol_default.clone();
        let range_default = range_default.clone();

        move |application_window| {
            let (width, height) = application_window.default_size();
            let maximized = application_window.is_maximized();
            let symbol_default = symbol_default.borrow();
            let symbol_default = symbol_default.as_str();
            let range_default = range_default.borrow();
            let range_default = range_default.as_api_str();

            settings.set_window_width(width);
            settings.set_window_height(height);
            settings.set_window_maximized(maximized);
            settings.set_last_stock(symbol_default);
            settings.set_last_range(range_default);

            Propagation::Proceed
        }
    });

    let (sender, mut receiver) = mpsc::channel(1);

    let split = NavigationSplitView::new();
    application_window.set_content(Some(&split));

    let sidebar = ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .build();
    let sidebar_page = create_sidebar_page(&sidebar, &settings, &sender, &range_default);
    split.set_sidebar(Some(&sidebar_page));

    let navigation = NavigationView::new();
    let placeholder_page = create_placeholder_page();
    navigation.push(&placeholder_page);
    let navigation_page = NavigationPage::new(&navigation, APP_NAME);
    split.set_content(Some(&navigation_page));

    navigation.connect_popped({
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        let symbol_default = symbol_default.clone();
        move |navigation, _| {
            let Some(visible_page) = navigation.visible_page() else {
                return;
            };

            if visible_page == placeholder_page {
                *symbol_default.borrow_mut() = String::new();
                sidebar.unselect_all();
            } else {
                let symbol = visible_page.title().to_string();
                *symbol_default.borrow_mut() = symbol.clone();
                if let Some(index) = settings.find_bookmark(&symbol) {
                    sidebar.select_row(sidebar.row_at_index(index as i32).as_ref());
                }
            }
        }
    });

    let breakpoint_condition = BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        540.0,
        LengthUnit::Sp,
    );
    let breakpoint = Breakpoint::new(breakpoint_condition);
    breakpoint.add_setter(&split, "collapsed", Some(&true.to_value()));
    application_window.add_breakpoint(breakpoint);

    let action_about = SimpleAction::new("about", None);
    action_about.connect_activate({
        let application_window = application_window.clone();
        move |_, _| {
            let about_dialog = create_about_dialog();
            about_dialog.present(Some(&application_window));
        }
    });
    application.add_action(&action_about);

    let action_add_bookmark = SimpleAction::new("add", None);
    action_add_bookmark.connect_activate({
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        let application_window = application_window.clone();
        move |_, _| {
            let add_dialog = create_add_dialog(&settings, &sidebar);
            add_dialog.present(Some(&application_window));
        }
    });
    application.add_action(&action_add_bookmark);

    if !last_stock.is_empty() {
        let runtime = runtime();
        runtime.spawn(Stock::fetch_and_send(last_stock, last_range, sender));
    }

    glib::spawn_future_local({
        let sidebar = sidebar.clone();
        async move {
            while let Some(stock) = receiver.recv().await {
                let symbol = stock.symbol.clone();
                *symbol_default.borrow_mut() = symbol.clone();

                let stock_page = create_stock_page(stock, &range_default);
                navigation.push(&stock_page);

                if let Some(index) = settings.find_bookmark(&symbol) {
                    sidebar.select_row(sidebar.row_at_index(index as i32).as_ref());
                }
            }
        }
    });

    application_window.present();
    sidebar.unselect_all();
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());

    runtime
}
